use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::spanned::Spanned;
use syn::{
    FnArg, Ident, ItemFn, Pat, PatType, ReturnType, Signature, Type, TypePath, parse_macro_input,
};

/// Extracted information about a resource injection parameter
struct ResourceInjection {
    var_name: Ident,
    full_type: Type,
    inner_type: TypePath,
    is_ref: bool,
    is_mut: bool,
}

/// Extracts the previous type and parameter name from function arguments,
fn extract_args_info(sig: &Signature) -> syn::Result<(Pat, TypePath, Vec<ResourceInjection>)> {
    if sig.inputs.is_empty() {
        return Err(syn::Error::new(
            sig.span(),
            "Chain function must have at least one parameter",
        ));
    }

    // First parameter: required, the previous chain type
    let first_arg = &sig.inputs[0];
    let (prev_param, previous_type) = match first_arg {
        FnArg::Typed(PatType { pat, ty, .. }) => {
            let param_pat = (**pat).clone();
            match &**ty {
                Type::Path(type_path) => (param_pat, type_path.clone()),
                _ => {
                    return Err(syn::Error::new(
                        ty.span(),
                        "First parameter type must be a type path",
                    ));
                }
            }
        }
        FnArg::Receiver(_) => {
            return Err(syn::Error::new(
                first_arg.span(),
                "Chain function cannot have self parameter",
            ));
        }
    };

    // 2nd to Nth parameters: optional, for resource injection
    let mut resources = Vec::new();
    for arg in sig.inputs.iter().skip(1) {
        match arg {
            FnArg::Typed(PatType { pat, ty, .. }) => {
                // Extract the variable name – must be a simple identifier
                let var_name = match &**pat {
                    Pat::Ident(pat_ident) => pat_ident.ident.clone(),
                    _ => {
                        return Err(syn::Error::new(
                            pat.span(),
                            "Resource injection parameter must be a simple identifier (e.g., `age: &Age`)",
                        ));
                    }
                };

                let full_type = *(*ty).clone();

                // Try to extract inner type for reference patterns like `&Age` -> `Age`
                // and `&mut Age` -> `Age`
                let (inner_type, is_ref, is_mut) = match &full_type {
                    Type::Reference(ref_type) => match &*ref_type.elem {
                        Type::Path(type_path) => {
                            let is_mut = ref_type.mutability.is_some();
                            (type_path.clone(), true, is_mut)
                        }
                        _ => {
                            return Err(syn::Error::new(
                                ty.span(),
                                "Reference resource type must be a type path (e.g., `age: &Age`)",
                            ));
                        }
                    },
                    Type::Path(type_path) => (type_path.clone(), false, false),
                    _ => {
                        return Err(syn::Error::new(
                            ty.span(),
                            "Resource injection type must be a type path or reference to one (e.g., `age: Age` or `age: &Age`)",
                        ));
                    }
                };

                resources.push(ResourceInjection {
                    var_name,
                    full_type,
                    inner_type,
                    is_ref,
                    is_mut,
                });
            }
            FnArg::Receiver(_) => {
                return Err(syn::Error::new(
                    arg.span(),
                    "Resource injection parameter cannot be self",
                ));
            }
        }
    }

    Ok((prev_param, previous_type, resources))
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
                    if last_segment.ident != "NextProcess" {
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

    // Extract the previous type, parameter name, and resource injection params
    let (prev_param, previous_type, resources) = match extract_args_info(&input_fn.sig) {
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

    // Check for async fn + &mut combination, which is not supported
    #[cfg(feature = "async")]
    if is_async_fn {
        if let Some(mut_res) = resources.iter().find(|r| r.is_mut) {
            return syn::Error::new(
                mut_res.var_name.span(),
                "Cannot use `&mut` resource injection in async chain function. ",
            )
            .to_compile_error()
            .into();
        }
    }

    // Separate resources into immutable refs and mutable refs
    let immut_resources: Vec<_> = resources.iter().filter(|r| !r.is_mut).collect();
    let mut_resources: Vec<_> = resources.iter().filter(|r| r.is_mut).collect();

    // Build resource injection statements for immutable references (let ... = ...)
    let immut_resource_stmts: Vec<_> = immut_resources
        .iter()
        .map(|res| {
            let var_binding_name = syn::Ident::new(
                &format!("{}_binding", &res.var_name.to_string()),
                res.var_name.span(),
            );
            let var_name = &res.var_name;
            let full_type = &res.full_type;
            let inner_type = &res.inner_type;
            if res.is_ref {
                quote! {
                    let #var_binding_name = ::mingling::this::<#program_type>()
                        .res_or_default::<#inner_type>();
                    let #var_name: #full_type = #var_binding_name.as_ref();
                }
            } else {
                quote! {
                    let #var_name: #full_type = ::mingling::this::<#program_type>()
                        .res_or_default::<#full_type>();
                }
            }
        })
        .collect();

    // Build nested __modify_res_and_return_any wrappers for mutable references.
    // The innermost layer is the original function body, wrapping outward for each
    // mutable resource.
    let body_stmts = &fn_body.stmts;
    let mut wrapped_body = quote! {
        #(#body_stmts)*
    };

    // Wrap from inside to outside: the first mutable parameter becomes the outermost wrapper,
    // and the last mutable parameter becomes the innermost wrapper.
    for res in mut_resources.iter() {
        let var_name = &res.var_name;
        let inner_type = &res.inner_type;
        wrapped_body = quote! {
            ::mingling::this::<#program_type>().__modify_res_and_return_any(|#var_name: &mut #inner_type| {
                #wrapped_body
            }).into()
        };
    }

    let has_immut_resources = !immut_resources.is_empty();
    let has_mut_resources = !mut_resources.is_empty();

    #[cfg(feature = "async")]
    let proc_fn = if is_async_fn {
        if has_immut_resources || has_mut_resources {
            quote! {
                async fn proc(#prev_param: #previous_type) -> ::mingling::ChainProcess<#program_type> {
                    #(#immut_resource_stmts)*
                    #wrapped_body
                }
            }
        } else {
            quote! {
                async fn proc(#prev_param: #previous_type) -> ::mingling::ChainProcess<#program_type> {
                    #fn_name(#prev_param).await.into()
                }
            }
        }
    } else {
        if has_immut_resources || has_mut_resources {
            quote! {
                async fn proc(#prev_param: #previous_type) -> ::mingling::ChainProcess<#program_type> {
                    #(#immut_resource_stmts)*
                    #wrapped_body
                }
            }
        } else {
            quote! {
                async fn proc(#prev_param: #previous_type) -> ::mingling::ChainProcess<#program_type> {
                    #fn_name(#prev_param).into()
                }
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
    let proc_fn = if has_immut_resources || has_mut_resources {
        quote! {
            fn proc(#prev_param: #previous_type) -> ::mingling::ChainProcess<#program_type> {
                #(#immut_resource_stmts)*
                #wrapped_body
            }
        }
    } else {
        quote! {
            fn proc(#prev_param: #previous_type) -> ::mingling::ChainProcess<#program_type> {
                #fn_name(#prev_param).into()
            }
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

    // Check that there are exactly two elements
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
