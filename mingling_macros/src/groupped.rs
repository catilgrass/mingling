use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Attribute, DeriveInput, Ident, parse_macro_input};

use crate::DEFAULT_PROGRAM_NAME;

/// Parses the `#[group(...)]` attribute to extract the group type
fn parse_group_attribute(attrs: &[Attribute]) -> Option<Ident> {
    for attr in attrs {
        if attr.path().is_ident("group")
            && let Ok(meta) = attr.parse_args::<syn::Meta>()
            && let syn::Meta::Path(path) = meta
            && let Some(segment) = path.segments.last()
        {
            return Some(segment.ident.clone());
        }
    }
    None
}

pub fn derive_groupped(input: TokenStream) -> TokenStream {
    // Parse the input struct/enum
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;

    // Parse attributes to find #[group(...)]
    let group_ident = parse_group_attribute(&input.attrs)
        .unwrap_or_else(|| Ident::new(DEFAULT_PROGRAM_NAME, Span::call_site()));

    let any_output_convert_impls = proc_macro2::TokenStream::from(build_any_output_convert_impls(
        struct_name.clone(),
        group_ident.clone(),
    ));

    // Generate the Groupped trait implementation
    let expanded = quote! {
        ::mingling::macros::register_type!(#struct_name);

        impl ::mingling::Groupped<#group_ident> for #struct_name {
            fn member_id() -> #group_ident {
                #group_ident::#struct_name
            }
        }

        #any_output_convert_impls
    };

    expanded.into()
}

#[cfg(feature = "general_renderer")]
pub fn derive_groupped_serialize(input: TokenStream) -> TokenStream {
    // Parse the input struct/enum
    let input_parsed = parse_macro_input!(input as DeriveInput);
    let struct_name = input_parsed.ident.clone();

    // Parse attributes to find #[group(...)]
    let group_ident = parse_group_attribute(&input_parsed.attrs)
        .unwrap_or_else(|| Ident::new(DEFAULT_PROGRAM_NAME, Span::call_site()));

    let any_output_convert_impls = proc_macro2::TokenStream::from(build_any_output_convert_impls(
        struct_name.clone(),
        group_ident.clone(),
    ));

    // Generate both Serialize and Groupped implementations
    let expanded = quote! {
        #[derive(serde::Serialize)]
        #input_parsed

        ::mingling::macros::register_type!(#struct_name);

        impl ::mingling::Groupped<#group_ident> for #struct_name {
            fn member_id() -> #group_ident {
                #group_ident::#struct_name
            }
        }

        #any_output_convert_impls
    };

    expanded.into()
}

fn build_any_output_convert_impls(struct_name: Ident, group_ident: Ident) -> TokenStream {
    quote! {
        impl ::std::convert::Into<::mingling::AnyOutput<#group_ident>> for #struct_name {
            fn into(self) -> ::mingling::AnyOutput<#group_ident> {
                ::mingling::AnyOutput::new(self)
            }
        }

        impl ::std::convert::Into<::mingling::ChainProcess<#group_ident>> for #struct_name {
            fn into(self) -> ::mingling::ChainProcess<#group_ident> {
                ::mingling::AnyOutput::new(self).route_chain()
            }
        }

        impl #struct_name {
            /// Converts the wrapper type into a `ChainProcess` for chaining operations.
            pub fn to_chain(self) -> ::mingling::ChainProcess<#group_ident> {
                ::mingling::AnyOutput::new(self).route_chain()
            }

            /// Converts the wrapper type into a `ChainProcess` for rendering operations.
            pub fn to_render(self) -> ::mingling::ChainProcess<#group_ident> {
                ::mingling::AnyOutput::new(self).route_renderer()
            }
        }
    }
    .into()
}
