use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Result as SynResult, Token, Type};

use crate::DEFAULT_PROGRAM_NAME;

enum PackInput {
    Explicit {
        group_name: Ident,
        type_name: Ident,
        inner_type: Type,
    },
    Default {
        type_name: Ident,
        inner_type: Type,
    },
}

impl Parse for PackInput {
    fn parse(input: ParseStream) -> SynResult<Self> {
        // Try to parse as explicit format first: GroupName, TypeName = InnerType
        let lookahead = input.lookahead1();

        if lookahead.peek(Ident) && input.peek2(Token![,]) {
            // Explicit format: GroupName, TypeName = InnerType
            let group_name = input.parse()?;
            input.parse::<Token![,]>()?;
            let type_name = input.parse()?;
            input.parse::<Token![=]>()?;
            let inner_type = input.parse()?;

            Ok(PackInput::Explicit {
                group_name,
                type_name,
                inner_type,
            })
        } else if lookahead.peek(Ident) && input.peek2(Token![=]) {
            // Default format: TypeName = InnerType
            let type_name = input.parse()?;
            input.parse::<Token![=]>()?;
            let inner_type = input.parse()?;

            Ok(PackInput::Default {
                type_name,
                inner_type,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

pub fn pack(input: TokenStream) -> TokenStream {
    let default_program_ident = crate::default_program_ident();

    // Parse the input
    let pack_input = syn::parse_macro_input!(input as PackInput);

    // Determine if we're using default or explicit group
    let (group_name, type_name, inner_type, use_default) = match pack_input {
        PackInput::Explicit {
            group_name,
            type_name,
            inner_type,
        } => (group_name, type_name, inner_type, false),
        PackInput::Default {
            type_name,
            inner_type,
        } => (
            Ident::new(DEFAULT_PROGRAM_NAME, proc_macro2::Span::call_site()),
            type_name,
            inner_type,
            true,
        ),
    };

    // Generate the struct definition
    #[cfg(not(feature = "general_renderer"))]
    let struct_def = quote! {
        pub struct #type_name {
            pub(crate) inner: #inner_type,
        }
    };

    #[cfg(feature = "general_renderer")]
    let struct_def = quote! {
        #[derive(serde::Serialize)]
        pub struct #type_name {
            pub(crate) inner: #inner_type,
        }
    };

    // Generate the new() method
    let new_impl = quote! {
        impl #type_name {
            /// Creates a new instance of the wrapper type
            pub fn new(inner: #inner_type) -> Self {
                Self { inner }
            }
        }
    };

    // Generate From and Into implementations
    let from_into_impl = quote! {
        impl From<#inner_type> for #type_name {
            fn from(inner: #inner_type) -> Self {
                Self::new(inner)
            }
        }

        impl From<#type_name> for #inner_type {
            fn from(wrapper: #type_name) -> #inner_type {
                wrapper.inner
            }
        }
    };

    // Generate AsRef and AsMut implementations
    let as_ref_impl = quote! {
        impl ::std::convert::AsRef<#inner_type> for #type_name {
            fn as_ref(&self) -> &#inner_type {
                &self.inner
            }
        }

        impl ::std::convert::AsMut<#inner_type> for #type_name {
            fn as_mut(&mut self) -> &mut #inner_type {
                &mut self.inner
            }
        }
    };

    // Generate Deref and DerefMut implementations
    let deref_impl = quote! {
        impl ::std::ops::Deref for #type_name {
            type Target = #inner_type;

            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }

        impl ::std::ops::DerefMut for #type_name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.inner
            }
        }
    };

    // Check if the inner type implements Default by generating conditional code
    let default_impl = quote! {
        impl ::std::default::Default for #type_name
        where
            #inner_type: ::std::default::Default,
        {
            fn default() -> Self {
                Self::new(::std::default::Default::default())
            }
        }
    };

    let register_impl = quote! {
        ::mingling::macros::register_type!(#type_name);
    };

    let any_out_impl = quote! {
        impl Into<mingling::AnyOutput<#group_name>> for #type_name {
            fn into(self) -> mingling::AnyOutput<#group_name> {
                mingling::AnyOutput::new(self)
            }
        }

        impl Into<mingling::ChainProcess<#group_name>> for #type_name {
            fn into(self) -> mingling::ChainProcess<#group_name> {
                mingling::AnyOutput::new(self).route_chain()
            }
        }

        impl #type_name {
            /// Converts the wrapper type into a `ChainProcess` for chaining operations.
            pub fn to_chain(self) -> mingling::ChainProcess<#group_name> {
                mingling::AnyOutput::new(self).route_chain()
            }

            /// Converts the wrapper type into a `ChainProcess` for rendering operations.
            pub fn to_render(self) -> mingling::ChainProcess<#group_name> {
                mingling::AnyOutput::new(self).route_renderer()
            }
        }
    };

    let group_impl = quote! {
        impl ::mingling::Groupped<#group_name> for #type_name {
            fn member_id() -> #group_name {
                #group_name::#type_name
            }
        }
    };

    // Combine all implementations
    let expanded = if use_default {
        // For default case, use ThisProgram
        quote! {
            #struct_def

            #new_impl
            #from_into_impl
            #as_ref_impl
            #deref_impl
            #default_impl
            #register_impl

            impl Into<mingling::AnyOutput<#default_program_ident>> for #type_name {
                fn into(self) -> mingling::AnyOutput<#default_program_ident> {
                    mingling::AnyOutput::new(self)
                }
            }

            impl From<#type_name> for mingling::ChainProcess<#default_program_ident> {
                fn from(value: #type_name) -> Self {
                    mingling::AnyOutput::new(value).route_chain()
                }
            }

            impl #type_name {
                /// Converts the wrapper type into a `ChainProcess` for chaining operations.
                pub fn to_chain(self) -> mingling::ChainProcess<#default_program_ident> {
                    mingling::AnyOutput::new(self).route_chain()
                }

                /// Converts the wrapper type into a `ChainProcess` for rendering operations.
                pub fn to_render(self) -> mingling::ChainProcess<#default_program_ident> {
                    mingling::AnyOutput::new(self).route_renderer()
                }
            }

            impl ::mingling::Groupped<#default_program_ident> for #type_name {
                fn member_id() -> #default_program_ident {
                    #default_program_ident::#type_name
                }
            }
        }
    } else {
        // For explicit case, use the provided group_name
        quote! {
            #struct_def

            #new_impl
            #from_into_impl
            #as_ref_impl
            #deref_impl
            #default_impl
            #register_impl

            #any_out_impl
            #group_impl
        }
    };

    expanded.into()
}
