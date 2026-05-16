use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{FnArg, Ident, ItemFn, PatType, Type, parse_macro_input};

#[cfg(feature = "comp")]
pub fn completion_attr(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the attribute arguments (e.g., HelloEntry from #[completion(HelloEntry)])
    let previous_type_ident = if attr.is_empty() {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "completion attribute requires a previous type argument, e.g. #[completion(HelloEntry)]",
        )
        .to_compile_error()
        .into();
    } else {
        parse_macro_input!(attr as Ident)
    };

    // Parse the function item
    let input_fn = parse_macro_input!(item as ItemFn);

    // Validate the function is not async
    if input_fn.sig.asyncness.is_some() {
        use syn::spanned::Spanned;

        return syn::Error::new(input_fn.sig.span(), "Completion function cannot be async")
            .to_compile_error()
            .into();
    }

    // Get the function signature parts
    let sig = &input_fn.sig;
    let inputs = &sig.inputs;
    let output = &sig.output;

    // Check that the function has exactly one parameter
    if inputs.len() != 1 {
        use syn::spanned::Spanned;

        return syn::Error::new(
            inputs.span(),
            "Completion function must have exactly one parameter",
        )
        .to_compile_error()
        .into();
    }

    // Check that the function parameter type is a single-segment type path (no `::`)
    if let Some(arg) = inputs.first()
        && let FnArg::Typed(PatType { ty, .. }) = arg
        && let Type::Path(type_path) = &**ty
        && type_path.path.segments.len() > 1
    {
        return syn::Error::new(
                        type_path.span(),
                        format!(
                            "The type `{}` in #[completion] function must be a simple single-segment type, e.g. `HelloEntry` instead of `other::HelloEntry`. Qualified paths with `::` are not allowed here.",
                            quote! { #type_path }
                        ),
                    )
                    .to_compile_error()
                    .into();
    }

    // Get the function body
    let fn_body = &input_fn.block;

    // Get function attributes (excluding the completion attribute)
    let mut fn_attrs = input_fn.attrs.clone();
    fn_attrs.retain(|attr| !attr.path().is_ident("completion"));

    // Get function visibility
    let vis = &input_fn.vis;

    // Get function name
    let fn_name = &sig.ident;

    // Generate internal name from function name using snake_case
    let internal_name = format!(
        "__internal_completion_{}",
        just_fmt::snake_case!(fn_name.to_string())
    );
    let struct_name = Ident::new(&internal_name, fn_name.span());

    // Generate the struct and implementation
    let expanded = quote! {
        #(#fn_attrs)*
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        #vis struct #struct_name;

        impl ::mingling::Completion for #struct_name {
            type Previous = #previous_type_ident;

            fn comp(#inputs) #output {
                #fn_body
            }
        }

        // Keep the original function for internal use
        #(#fn_attrs)*
        #vis fn #fn_name(#inputs) #output {
            #fn_body
        }
    };

    let completion_entry = quote! {
        Self::#previous_type_ident => <#struct_name as ::mingling::Completion>::comp(ctx),
    };

    let mut completions = crate::COMPLETIONS.lock().unwrap();
    let completion_str = completion_entry.to_string();
    completions.insert(completion_str);

    expanded.into()
}
