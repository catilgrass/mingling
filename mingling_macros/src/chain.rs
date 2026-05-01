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
            "Chain function must have exactly one parameter",
        ));
    }

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
            "Chain function cannot have self parameter",
        )),
    }
}

pub fn chain_attr(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the attribute arguments (e.g., MyProgram from #[chain(MyProgram)])
    // If no argument is provided, use ThisProgram
    let (group_name, use_crate_prefix) = if attr.is_empty() {
        (
            Ident::new("ThisProgram", proc_macro2::Span::call_site()),
            true,
        )
    } else {
        (parse_macro_input!(attr as Ident), false)
    };

    // Parse the function item
    let input_fn = parse_macro_input!(item as ItemFn);

    // In `async` mode, check if the function is an async function
    #[cfg(feature = "async")]
    let is_async_fn = input_fn.sig.asyncness.is_some();

    // Validate the chain functions is a regular function
    #[cfg(not(feature = "async"))]
    {
        if input_fn.sig.asyncness.is_some() {
            return syn::Error::new(
                input_fn.sig.span(),
                "Chain function cannot be async when async feature is disabled",
            )
            .to_compile_error()
            .into();
        }
    }

    // Check that return type is NextProcess
    let return_type = &input_fn.sig.output;
    match return_type {
        ReturnType::Type(_, ty) => {
            // Check if the return type is NextProcess
            match &**ty {
                Type::Path(type_path) => {
                    let last_segment = type_path.path.segments.last().unwrap();
                    if last_segment.ident.to_string() != "NextProcess" {
                        return syn::Error::new(
                            ty.span(),
                            "Chain function must return `NextProcess`",
                        )
                        .to_compile_error()
                        .into();
                    }
                }
                _ => {
                    return syn::Error::new(ty.span(), "Chain function must return `NextProcess`")
                        .to_compile_error()
                        .into();
                }
            }
        }
        ReturnType::Default => {
            return syn::Error::new(
                input_fn.sig.span(),
                "Chain function must specify a return type (must be `NextProcess`)",
            )
            .to_compile_error()
            .into();
        }
    }

    // Extract the previous type and parameter name from function arguments
    let (prev_param, previous_type) = match extract_previous_info(&input_fn.sig) {
        Ok(info) => info,
        Err(e) => return e.to_compile_error().into(),
    };

    // Get the function signature components for direct substitution
    let sig = &input_fn.sig;
    let inputs = &sig.inputs;

    // Get the function body
    let fn_body = &input_fn.block;

    // Get function attributes (excluding the chain attribute)
    let mut fn_attrs = input_fn.attrs.clone();

    // Remove any #[chain(...)] attributes to avoid infinite recursion
    fn_attrs.retain(|attr| !attr.path().is_ident("chain"));

    // Get function visibility
    let vis = &input_fn.vis;

    // Get function name
    let fn_name = &input_fn.sig.ident;

    // Generate struct name from function name using snake_case
    let internal_name = format!(
        "__internal_chain_{}",
        just_fmt::snake_case!(fn_name.to_string())
    );
    let struct_name = Ident::new(&internal_name, fn_name.span());

    // Determine the program type for the return type
    let program_type = if use_crate_prefix {
        quote! { ThisProgram }
    } else {
        quote! { #group_name }
    };

    #[cfg(feature = "async")]
    let proc_fn = if is_async_fn {
        quote! {
            async fn proc(#inputs) -> ::mingling::ChainProcess<#program_type> {
                #fn_name(#prev_param).await.into()
            }
        }
    } else {
        quote! {
            async fn proc(#inputs) -> ::mingling::ChainProcess<#program_type> {
                #fn_name(#prev_param).into()
            }
        }
    };

    #[cfg(feature = "async")]
    let origin_proc_fn = if is_async_fn {
        quote! {
            #(#fn_attrs)*
            #vis async fn #fn_name(#inputs) -> impl Into<::mingling::ChainProcess<#program_type>> {
                #fn_body
            }
        }
    } else {
        quote! {
            #(#fn_attrs)*
            #vis fn #fn_name(#inputs) -> impl Into<::mingling::ChainProcess<#program_type>> {
                #fn_body
            }
        }
    };

    #[cfg(not(feature = "async"))]
    let proc_fn = quote! {
        fn proc(#inputs) -> ::mingling::ChainProcess<#program_type> {
            #fn_name(#prev_param).into()
        }
    };

    #[cfg(not(feature = "async"))]
    let origin_proc_fn = quote! {
        #(#fn_attrs)*
        #vis fn #fn_name(#inputs) -> impl Into<::mingling::ChainProcess<#program_type>> {
            #fn_body
        }
    };

    // Generate the struct and implementation
    let expanded = if use_crate_prefix {
        quote! {
            #(#fn_attrs)*
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            #vis struct #struct_name;

            ::mingling::macros::register_chain!(#previous_type, #struct_name);

            impl ::mingling::Chain<ThisProgram> for #struct_name {
                type Previous = #previous_type;

                #proc_fn
            }

            // Keep the original function for internal use
            #origin_proc_fn
        }
    } else {
        quote! {
            #(#fn_attrs)*
            #[allow(non_camel_case_types)]
            #vis struct #struct_name;

            ::mingling::macros::register_chain!(#previous_type, #struct_name);

            impl ::mingling::Chain<#group_name> for #struct_name {
                type Previous = #previous_type;

                #proc_fn
            }

            // Keep the original function for internal use
            #origin_proc_fn
        }
    };

    expanded.into()
}

/// Builds a match arm for chain mapping
pub fn build_chain_arm(struct_name: &Ident, previous_type: &TypePath) -> proc_macro2::TokenStream {
    quote! {
        #struct_name => #previous_type,
    }
}

/// Builds a match arm for chain existence check
pub fn build_chain_exist_arm(previous_type: &TypePath) -> proc_macro2::TokenStream {
    quote! {
        Self::#previous_type => true,
    }
}

pub fn register_chain(input: TokenStream) -> TokenStream {
    // Parse the input as a comma-separated list of arguments
    let input_parsed = syn::parse_macro_input!(input with syn::punctuated::Punctuated<syn::Expr, syn::Token![,]>::parse_terminated);

    // Check that we have exactly two elements
    if input_parsed.len() != 2 {
        return syn::Error::new(
            input_parsed.span(),
            "Expected exactly two comma-separated arguments: `PreviousType, StructName`",
        )
        .to_compile_error()
        .into();
    }

    // Extract the two elements
    let previous_type_expr = &input_parsed[0];
    let struct_name_expr = &input_parsed[1];

    // Convert expressions to TypePath and Ident
    let previous_type = match syn::parse2::<TypePath>(previous_type_expr.to_token_stream()) {
        Ok(ty) => ty,
        Err(e) => return e.to_compile_error().into(),
    };

    let struct_name = match syn::parse2::<syn::Ident>(struct_name_expr.to_token_stream()) {
        Ok(ident) => ident,
        Err(e) => return e.to_compile_error().into(),
    };

    // Record the chain mapping: previous_type => struct_name
    let chain_entry = build_chain_arm(&struct_name, &previous_type);

    // Record the chain existence check
    let chain_exist_entry = build_chain_exist_arm(&previous_type);

    let mut chains = crate::CHAINS.lock().unwrap();
    let mut chain_exist = crate::CHAINS_EXIST.lock().unwrap();

    let chain_entry_str = chain_entry.to_string();
    let chain_exist_entry_str = chain_exist_entry.to_string();

    chains.insert(chain_entry_str);
    chain_exist.insert(chain_exist_entry_str);

    quote! {}.into()
}
