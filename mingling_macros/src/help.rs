use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::spanned::Spanned;
use syn::{
    FnArg, Ident, ItemFn, Pat, PatType, ReturnType, Signature, Type, TypePath, parse_macro_input,
};

/// Extracts the previous type and parameter name from function arguments
fn extract_previous_info(sig: &Signature) -> syn::Result<(Pat, TypePath)> {
    // The function should have exactly one parameter
    if sig.inputs.len() != 1 {
        return Err(syn::Error::new(
            sig.inputs.span(),
            "Help function must have exactly one parameter (the entry type)",
        ));
    }

    // First and only parameter is the entry type
    let arg = &sig.inputs[0];
    match arg {
        FnArg::Typed(PatType { pat, ty, .. }) => {
            // Extract the pattern (parameter name)
            let param_pat = (**pat).clone();

            // Extract the type
            match &**ty {
                Type::Path(type_path) => Ok((param_pat, type_path.clone())),
                _ => Err(syn::Error::new(
                    ty.span(),
                    "Parameter type must be a type path",
                )),
            }
        }
        FnArg::Receiver(_) => Err(syn::Error::new(
            arg.span(),
            "Help function cannot have self parameter",
        )),
    }
}

/// Validates the return type is () or empty
fn validate_return_type(sig: &Signature) -> syn::Result<()> {
    match &sig.output {
        ReturnType::Type(_, ty) => match &**ty {
            Type::Tuple(tuple) if tuple.elems.is_empty() => Ok(()),
            _ => Err(syn::Error::new(
                ty.span(),
                "Help function must return () or have no return type",
            )),
        },
        ReturnType::Default => Ok(()),
    }
}

pub fn help_attr(item: TokenStream) -> TokenStream {
    // Parse the function item
    let input_fn = parse_macro_input!(item as ItemFn);

    // Validate the function is not async
    if input_fn.sig.asyncness.is_some() {
        return syn::Error::new(input_fn.sig.span(), "Help function cannot be async")
            .to_compile_error()
            .into();
    }

    // Extract the entry type and parameter name from function arguments
    let (prev_param, entry_type) = match extract_previous_info(&input_fn.sig) {
        Ok(info) => info,
        Err(e) => return e.to_compile_error().into(),
    };

    // Check that the entry type is a single-segment type (no `::`)
    if let Some(err_tokens) = crate::check_single_segment_type(&entry_type, "#[help]") {
        return err_tokens.into();
    }

    // Validate return type
    if let Err(e) = validate_return_type(&input_fn.sig) {
        return e.to_compile_error().into();
    }

    // Get the function body
    let fn_body = &input_fn.block;

    // Get function attributes (excluding the help attribute)
    let mut fn_attrs = input_fn.attrs.clone();
    fn_attrs.retain(|attr| !attr.path().is_ident("help"));

    // Get function visibility
    let vis = &input_fn.vis;

    // Get function name
    let fn_name = &input_fn.sig.ident;

    // Generate internal name using snake_case for the chain macro
    let internal_name = format!(
        "__internal_help_{}",
        just_fmt::snake_case!(fn_name.to_string())
    );
    let struct_name = Ident::new(&internal_name, fn_name.span());

    // Register the help request mapping
    let help_entry = build_help_entry(&struct_name, &entry_type);
    let entry_str = help_entry.to_string();
    crate::HELP_REQUESTS.lock().unwrap().insert(entry_str);

    // Generate the struct and HelpRequest implementation
    let expanded = quote! {
        #(#fn_attrs)*
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        #vis struct #struct_name;

        impl ::mingling::HelpRequest for #struct_name {
            type Entry = #entry_type;

            fn render_help(#prev_param: Self::Entry, r: &mut ::mingling::RenderResult) {
                // Create a local wrapper function that includes r parameter
                // This allows r_println! to access r
                #[allow(non_snake_case)]
                fn help_wrapper(#prev_param: #entry_type, r: &mut ::mingling::RenderResult) {
                    #fn_body
                }

                // Call the wrapper function
                help_wrapper(#prev_param, r);
            }
        }

        ::mingling::macros::register_help!(#entry_type, #struct_name);

        // Keep the original function for internal use (without r parameter)
        #(#fn_attrs)*
        #vis fn #fn_name(#prev_param: #entry_type) {
            let mut dummy_r = ::mingling::RenderResult::default();
            let r = &mut dummy_r;
            #fn_body
        }
    };

    expanded.into()
}

/// Builds a help request entry for the global help requests list
fn build_help_entry(struct_name: &Ident, entry_type: &TypePath) -> proc_macro2::TokenStream {
    let enum_variant = &entry_type.path.segments.last().unwrap().ident;
    quote! {
        Self::#enum_variant => {
            // SAFETY: The member_id check ensures that `any` contains a value of type `#entry_type`,
            // so downcasting to `#entry_type` is safe.
            let value = unsafe { any.downcast::<#entry_type>().unwrap_unchecked() };
            <#struct_name as ::mingling::HelpRequest>::render_help(value, r);
        }
    }
}

pub fn register_help(input: TokenStream) -> TokenStream {
    // Parse the input as a comma-separated list of arguments
    let input_parsed = syn::parse_macro_input!(input with syn::punctuated::Punctuated<syn::Expr, syn::Token![,]>::parse_terminated);

    // Check that we have exactly two elements
    if input_parsed.len() != 2 {
        return syn::Error::new(
            input_parsed.span(),
            "Expected exactly two comma-separated arguments: `EntryType, StructName`",
        )
        .to_compile_error()
        .into();
    }

    // Extract the two elements
    let entry_type_expr = &input_parsed[0];
    let struct_name_expr = &input_parsed[1];

    // Convert expressions to TypePath and Ident
    let entry_type = match syn::parse2::<TypePath>(entry_type_expr.to_token_stream()) {
        Ok(ty) => ty,
        Err(e) => return e.to_compile_error().into(),
    };

    let struct_name = match syn::parse2::<syn::Ident>(struct_name_expr.to_token_stream()) {
        Ok(ident) => ident,
        Err(e) => return e.to_compile_error().into(),
    };

    // Register the help request mapping
    let help_entry = build_help_entry(&struct_name, &entry_type);
    let entry_str = help_entry.to_string();
    crate::HELP_REQUESTS.lock().unwrap().insert(entry_str);

    quote! {}.into()
}
