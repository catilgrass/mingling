//! Renderer Attribute Macro Implementation
//!
//! This module provides the `#[renderer]` attribute macro for automatically
//! generating structs that implement the `Renderer` trait from functions.

use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::spanned::Spanned;
use syn::{FnArg, ItemFn, Pat, PatType, ReturnType, Signature, Type, TypePath, parse_macro_input};

/// Extracts the previous type and parameter name from function arguments
fn extract_previous_info(sig: &Signature) -> syn::Result<(Pat, TypePath)> {
    // The function should have exactly one parameter
    if sig.inputs.len() != 1 {
        return Err(syn::Error::new(
            sig.inputs.span(),
            "Renderer function must have exactly one parameter (the previous type)",
        ));
    }

    // First and only parameter is the previous type
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
            "Renderer function cannot have self parameter",
        )),
    }
}

/// Extracts the return type from the function signature
fn extract_return_type(sig: &Signature) -> syn::Result<()> {
    // Renderer functions should return () or have no return type
    match &sig.output {
        ReturnType::Type(_, ty) => {
            // Check if it's ()
            match &**ty {
                Type::Tuple(tuple) if tuple.elems.is_empty() => Ok(()),
                _ => Err(syn::Error::new(
                    ty.span(),
                    "Renderer function must return () or have no return type",
                )),
            }
        }
        ReturnType::Default => Ok(()),
    }
}

pub fn renderer_attr(item: TokenStream) -> TokenStream {
    // Parse the function item
    let input_fn = parse_macro_input!(item as ItemFn);

    // Validate the function is not async
    if input_fn.sig.asyncness.is_some() {
        return syn::Error::new(input_fn.sig.span(), "Renderer function cannot be async")
            .to_compile_error()
            .into();
    }

    // Extract the previous type and parameter name from function arguments
    let (prev_param, previous_type) = match extract_previous_info(&input_fn.sig) {
        Ok(info) => info,
        Err(e) => return e.to_compile_error().into(),
    };

    // Validate return type
    if let Err(e) = extract_return_type(&input_fn.sig) {
        return e.to_compile_error().into();
    }

    // Get the function body
    let fn_body = &input_fn.block;

    // Get function attributes (excluding the renderer attribute)
    let mut fn_attrs = input_fn.attrs.clone();

    // Remove any #[renderer(...)] attributes to avoid infinite recursion
    fn_attrs.retain(|attr| !attr.path().is_ident("renderer"));

    // Get function visibility
    let vis = &input_fn.vis;

    // Get function name
    let fn_name = &input_fn.sig.ident;

    // Generate struct name from function name using pascal_case
    let internal_name = format!(
        "__internal_renderer_{}",
        just_fmt::snake_case!(fn_name.to_string())
    );
    let struct_name = syn::Ident::new(&internal_name, fn_name.span());

    // Generate the struct and implementation
    // We need to create a wrapper function that adds the r parameter
    let expanded = quote! {
        #(#fn_attrs)*
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        #vis struct #struct_name;

        ::mingling::macros::register_renderer!(#previous_type, #struct_name);

        impl ::mingling::Renderer for #struct_name {
            type Previous = #previous_type;

            fn render(#prev_param: Self::Previous, r: &mut ::mingling::RenderResult) {
                // Create a local wrapper function that includes r parameter
                // This allows r_println! to access r
                #[allow(non_snake_case)]
                fn render_wrapper(#prev_param: #previous_type, r: &mut ::mingling::RenderResult) {
                    #fn_body
                }

                // Call the wrapper function
                render_wrapper(#prev_param, r);
            }
        }

        // Keep the original function for internal use (without r parameter)
        #(#fn_attrs)*
        #vis fn #fn_name(#prev_param: impl Into<#previous_type>) -> ::mingling::RenderResult {
            let #prev_param = #prev_param.into();
            let mut dummy_r = ::mingling::RenderResult::default();
            {
                let r = &mut dummy_r;
                #fn_body
            }
            dummy_r
        }
    };

    expanded.into()
}

/// Builds the renderer entry for the global renderers list
pub fn build_renderer_entry(
    struct_name: &syn::Ident,
    previous_type: &TypePath,
) -> proc_macro2::TokenStream {
    quote! {
        #struct_name => #previous_type,
    }
}

/// Builds the renderer existence check entry
pub fn build_renderer_exist_entry(previous_type: &TypePath) -> proc_macro2::TokenStream {
    quote! {
        Self::#previous_type => true,
    }
}

/// Builds the general renderer entry
#[cfg(feature = "general_renderer")]
pub fn build_general_renderer_entry(previous_type: &TypePath) -> proc_macro2::TokenStream {
    quote! {
        Self::#previous_type => {
            // SAFETY: Only types that match will enter this branch for forced conversion,
            // and `AnyOutput::new` ensures the type implements serde::Serialize
            let raw = unsafe { any.restore::<#previous_type>().unwrap_unchecked() };
            let mut r = ::mingling::RenderResult::default();
            ::mingling::GeneralRenderer::render(&raw, setting, &mut r)?;
            Ok(r)
        }
    }
}

pub fn register_renderer(input: TokenStream) -> TokenStream {
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

    // Register the renderer in the global list
    let renderer_entry = build_renderer_entry(&struct_name, &previous_type);
    let renderer_exist_entry = build_renderer_exist_entry(&previous_type);
    #[cfg(feature = "general_renderer")]
    let general_renderer_entry = build_general_renderer_entry(&previous_type);

    let mut renderers = crate::RENDERERS.lock().unwrap();
    let mut renderer_exist = crate::RENDERERS_EXIST.lock().unwrap();

    #[cfg(feature = "general_renderer")]
    let mut general_renderers = crate::GENERAL_RENDERERS.lock().unwrap();

    let renderer_entry_str = renderer_entry.to_string();
    let renderer_exist_entry_str = renderer_exist_entry.to_string();

    #[cfg(feature = "general_renderer")]
    let general_renderer_entry_str = general_renderer_entry.to_string();

    renderers.insert(renderer_entry_str);
    renderer_exist.insert(renderer_exist_entry_str);

    #[cfg(feature = "general_renderer")]
    general_renderers.insert(general_renderer_entry_str);

    quote! {}.into()
}
