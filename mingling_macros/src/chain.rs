#![allow(clippy::too_many_arguments)]

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

    // First parameter: required, the previous chain type (must be owned, not a reference)
    let first_arg = &sig.inputs[0];
    let (prev_param, previous_type) = match first_arg {
        FnArg::Typed(PatType { pat, ty, .. }) => {
            let param_pat = (**pat).clone();
            match &**ty {
                Type::Path(type_path) => {
                    // Check that the type is a single-segment type (no `::`)
                    if type_path.path.segments.len() > 1 {
                        return Err(syn::Error::new(
                            type_path.span(),
                            format!(
                                "The type `{}` in #[chain] function must be a simple single-segment type, \
                                 e.g. `Empty` instead of `other::Empty`. \
                                 Qualified paths with `::` are not allowed here.",
                                quote! { #type_path }
                            ),
                        ));
                    }
                    (param_pat, type_path.clone())
                }
                Type::Reference(_) => {
                    return Err(syn::Error::new(
                        ty.span(),
                        "The first parameter (previous type) must be taken by move, \
                         not by reference. \
                         Use `prev: SomeEntry` instead of `prev: &SomeEntry`.",
                    ));
                }
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
                    Type::Path(_) => {
                        return Err(syn::Error::new(
                            ty.span(),
                            "Resource injection parameter must be a reference (`&T` or `&mut T`), \
                             not an owned value. Use `age: &Age` instead of `age: Age`.",
                        ));
                    }
                    _ => {
                        return Err(syn::Error::new(
                            ty.span(),
                            "Resource injection type must be a type path or reference to one \
                             (e.g., `age: Age` or `age: &Age`)",
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

/// Parses the `#[chain(...)]` attribute arguments.
///
/// Returns:
/// - `program_path`: the token stream representing the program type
/// - `use_crate_prefix`: whether to use the default crate-defined program path
fn parse_chain_attr_args(attr: TokenStream) -> (proc_macro2::TokenStream, bool) {
    if attr.is_empty() {
        (crate::default_program_path(), true)
    } else {
        let path: syn::Path = syn::parse(attr).expect("#[chain(..)] argument must be a path");
        (quote! { #path }, false)
    }
}

/// Validates that the return type of the function is `NextProcess`.
fn validate_return_type_is_next_process(sig: &Signature) -> Result<(), proc_macro2::TokenStream> {
    match &sig.output {
        ReturnType::Type(_, ty) => match &**ty {
            Type::Path(type_path) => {
                let last_segment = type_path.path.segments.last().unwrap();
                if last_segment.ident != "NextProcess" {
                    return Err(syn::Error::new(
                        ty.span(),
                        "Chain function must return `NextProcess`",
                    )
                    .to_compile_error());
                }
            }
            _ => {
                return Err(
                    syn::Error::new(ty.span(), "Chain function must return `NextProcess`")
                        .to_compile_error(),
                );
            }
        },
        ReturnType::Default => {
            return Err(syn::Error::new(
                sig.span(),
                "Chain function must specify a return type (must be `NextProcess`)",
            )
            .to_compile_error());
        }
    }
    Ok(())
}

/// Generates `let` binding statements for immutable resource injection parameters.
///
/// Each immutable reference parameter gets a `_binding` variable that holds the
/// `res_or_default` result, then a shadowing `let` that borrows from it via `.as_ref()`.
fn generate_immut_resource_bindings<'a>(
    resources: impl Iterator<Item = &'a ResourceInjection>,
    program_type: &proc_macro2::TokenStream,
) -> Vec<proc_macro2::TokenStream> {
    resources
        .filter(|r| !r.is_mut)
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
        .collect()
}

/// Wraps the function body in nested `__modify_res_and_return_any` closures for
/// each mutable resource parameter. The innermost closure gets the original body,
/// and each mutable parameter wraps outward from last to first.
fn wrap_body_with_mut_resources(
    fn_body_stmts: &[syn::Stmt],
    mut_resources: &[&ResourceInjection],
    program_type: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let mut wrapped = quote! {
        #(#fn_body_stmts)*
    };

    for res in mut_resources.iter() {
        let var_name = &res.var_name;
        let inner_type = &res.inner_type;
        wrapped = quote! {
            ::mingling::this::<#program_type>().__modify_res_and_return_any(|#var_name: &mut #inner_type| {
                #wrapped
            }).into()
        };
    }

    wrapped
}

/// Builds the `proc` function implementation that serves as the actual chain
/// entry point inside the generated `Chain` impl.
///
/// * Without resources: delegates directly to the original function.
/// * With resources: inlines the body and prepends resource bindings.
#[allow(unused_variables)]
fn generate_proc_fn(
    has_resources: bool,
    resources: &[ResourceInjection],
    program_type: &proc_macro2::TokenStream,
    previous_type: &TypePath,
    prev_param: &Pat,
    fn_name: &Ident,
    fn_body_stmts: &[syn::Stmt],
    is_async_fn: bool,
) -> proc_macro2::TokenStream {
    let immut_resource_stmts = generate_immut_resource_bindings(resources.iter(), program_type);
    let mut_resources: Vec<_> = resources.iter().filter(|r| r.is_mut).collect();
    let wrapped_body = wrap_body_with_mut_resources(fn_body_stmts, &mut_resources, program_type);

    #[cfg(feature = "async")]
    {
        if has_resources {
            quote! {
                async fn proc(#prev_param: #previous_type) -> ::mingling::ChainProcess<#program_type> {
                    #(#immut_resource_stmts)*
                    #wrapped_body
                }
            }
        } else {
            let call = if is_async_fn {
                quote! { #fn_name(#prev_param).await.into() }
            } else {
                quote! { #fn_name(#prev_param).into() }
            };
            quote! {
                async fn proc(#prev_param: #previous_type) -> ::mingling::ChainProcess<#program_type> {
                    #call
                }
            }
        }
    }

    #[cfg(not(feature = "async"))]
    {
        if has_resources {
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
        }
    }
}

/// Generates the original function signature (kept for backwards compatibility /
/// internal use), with its return type changed to `impl Into<ChainProcess<..>>`.
#[allow(unused_variables)]
fn generate_original_fn(
    fn_attrs: &[syn::Attribute],
    vis: &syn::Visibility,
    fn_name: &Ident,
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    fn_body: &syn::Block,
    is_async_fn: bool,
    program_type: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    #[cfg(feature = "async")]
    {
        let async_kw = if is_async_fn {
            quote! { async }
        } else {
            quote! {}
        };
        quote! {
            #(#fn_attrs)*
            #vis #async_kw fn #fn_name(#inputs) -> impl Into<::mingling::ChainProcess<#program_type>> {
                #fn_body
            }
        }
    }

    #[cfg(not(feature = "async"))]
    {
        quote! {
            #(#fn_attrs)*
            #vis fn #fn_name(#inputs) -> impl Into<::mingling::ChainProcess<#program_type>> {
                #fn_body
            }
        }
    }
}

/// Assembles the final expanded output: hidden struct, `register_chain!` invocation,
/// `Chain` impl with the `proc` method, and the original function.
fn generate_struct_and_impl(
    fn_attrs: &[syn::Attribute],
    vis: &syn::Visibility,
    struct_name: &Ident,
    previous_type: &TypePath,
    previous_type_str: &proc_macro2::TokenStream,
    group_name: &proc_macro2::TokenStream,
    program_type: &proc_macro2::TokenStream,
    use_crate_prefix: bool,
    proc_fn: proc_macro2::TokenStream,
    origin_proc_fn: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let chain_type = if use_crate_prefix {
        program_type
    } else {
        group_name
    };

    quote! {
        #(#fn_attrs)*
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        #vis struct #struct_name;

        ::mingling::macros::register_chain!(#previous_type_str, #struct_name);

        impl ::mingling::Chain<#chain_type> for #struct_name {
            type Previous = #previous_type;

            #proc_fn
        }

        // Keep the original function for internal use
        #origin_proc_fn
    }
}

/// Ensures the function is not async when the `async` feature is disabled.
#[cfg(not(feature = "async"))]
fn reject_async(sig: &Signature) -> Result<(), proc_macro2::TokenStream> {
    if sig.asyncness.is_some() {
        return Err(syn::Error::new(
            sig.span(),
            "Chain function cannot be async when async feature is disabled",
        )
        .to_compile_error());
    }
    Ok(())
}

/// Ensures no `&mut` resource injection is used in async functions.
#[cfg(feature = "async")]
fn reject_mut_in_async(resources: &[ResourceInjection]) -> Result<(), proc_macro2::TokenStream> {
    if let Some(mut_res) = resources.iter().find(|r| r.is_mut) {
        return Err(syn::Error::new(
            mut_res.var_name.span(),
            "Cannot use `&mut` resource injection in async chain function.",
        )
        .to_compile_error());
    }
    Ok(())
}

pub fn chain_attr(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse attribute arguments
    let (group_name, use_crate_prefix) = parse_chain_attr_args(attr);

    // Parse the function item
    let input_fn = parse_macro_input!(item as ItemFn);

    // Handle async feature gate
    #[cfg(feature = "async")]
    let is_async_fn = input_fn.sig.asyncness.is_some();

    #[cfg(not(feature = "async"))]
    {
        if let Err(err) = reject_async(&input_fn.sig) {
            return err.into();
        }
    }

    // Validate return type
    if let Err(err) = validate_return_type_is_next_process(&input_fn.sig) {
        return err.into();
    }

    // Extract the previous type, parameter name, and resource injection params
    let (prev_param, previous_type, resources) = match extract_args_info(&input_fn.sig) {
        Ok(info) => info,
        Err(e) => return e.to_compile_error().into(),
    };

    // Reject `&mut` in async chains
    #[cfg(feature = "async")]
    if is_async_fn {
        if let Err(err) = reject_mut_in_async(&resources) {
            return err.into();
        }
    }

    // Prepare building blocks
    let sig = &input_fn.sig;
    let inputs = &sig.inputs;
    let fn_body = &input_fn.block;
    let mut fn_attrs = input_fn.attrs.clone();
    fn_attrs.retain(|attr| !attr.path().is_ident("chain"));
    let vis = &input_fn.vis;
    let fn_name = &input_fn.sig.ident;
    let has_resources = !resources.is_empty();

    // Generate struct name
    let internal_name = format!(
        "__internal_chain_{}",
        just_fmt::snake_case!(fn_name.to_string())
    );
    let struct_name = Ident::new(&internal_name, fn_name.span());

    // Determine the program type for the return type
    let program_type = if use_crate_prefix {
        crate::default_program_path()
    } else {
        group_name.clone()
    };

    // Generate the `proc` function
    let proc_fn = generate_proc_fn(
        has_resources,
        &resources,
        &program_type,
        &previous_type,
        &prev_param,
        fn_name,
        &fn_body.stmts,
        #[cfg(feature = "async")]
        is_async_fn,
        #[cfg(not(feature = "async"))]
        false,
    );

    // Generate the original function
    let origin_proc_fn = generate_original_fn(
        &fn_attrs,
        vis,
        fn_name,
        inputs,
        fn_body,
        #[cfg(feature = "async")]
        is_async_fn,
        #[cfg(not(feature = "async"))]
        false,
        &program_type,
    );

    // Assemble the final output
    let previous_type_str = quote! { #previous_type };
    let expanded = generate_struct_and_impl(
        &fn_attrs,
        vis,
        &struct_name,
        &previous_type,
        &previous_type_str,
        &group_name,
        &program_type,
        use_crate_prefix,
        proc_fn,
        origin_proc_fn,
    );

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
