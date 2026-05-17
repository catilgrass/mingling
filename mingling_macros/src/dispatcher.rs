use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Result as SynResult, Token};

#[cfg(feature = "dispatch_tree")]
use crate::COMPILE_TIME_DISPATCHERS;

enum DispatcherChainInput {
    Explicit {
        group_name: syn::Path,
        command_name: syn::LitStr,
        command_struct: Ident,
        pack: Ident,
    },
    Default {
        command_name: syn::LitStr,
        command_struct: Ident,
        pack: Ident,
    },
}

impl Parse for DispatcherChainInput {
    fn parse(input: ParseStream) -> SynResult<Self> {
        if (input.peek(Ident) || input.peek(Token![crate]))
            && (input.peek2(Token![::]) || input.peek2(Token![,]))
        {
            let group_name = input.parse::<syn::Path>()?;
            input.parse::<Token![,]>()?;
            let command_name = input.parse()?;
            input.parse::<Token![,]>()?;
            let command_struct = input.parse()?;
            input.parse::<Token![=>]>()?;
            let pack = input.parse()?;

            Ok(DispatcherChainInput::Explicit {
                group_name,
                command_name,
                command_struct,
                pack,
            })
        } else if input.peek(syn::LitStr) {
            // Default format: "command_name", CommandStruct => ChainStruct
            let command_name = input.parse()?;
            input.parse::<Token![,]>()?;
            let command_struct = input.parse()?;
            input.parse::<Token![=>]>()?;
            let pack = input.parse()?;

            Ok(DispatcherChainInput::Default {
                command_name,
                command_struct,
                pack,
            })
        } else {
            Err(input.lookahead1().error())
        }
    }
}

// NOTICE: This implementation contains significant code duplication between the explicit
// and default cases in both `dispatcher_chain` and `dispatcher_render` functions.
// The logic for handling default vs explicit group names and generating the appropriate
// code should be extracted into common helper functions to reduce redundancy.
// Additionally, the token stream generation patterns are nearly identical between
// the two main functions and could benefit from refactoring.

pub fn dispatcher(input: TokenStream) -> TokenStream {
    // Parse the input
    let dispatcher_input = syn::parse_macro_input!(input as DispatcherChainInput);

    // Determine if we're using default or explicit group
    let (command_name, command_struct, pack, _use_default, group_path) = match dispatcher_input {
        DispatcherChainInput::Explicit {
            group_name,
            command_name,
            command_struct,
            pack,
        } => (
            command_name,
            command_struct,
            pack,
            false,
            quote! { #group_name },
        ),
        DispatcherChainInput::Default {
            command_name,
            command_struct,
            pack,
        } => (
            command_name,
            command_struct,
            pack,
            true,
            crate::default_program_path(),
        ),
    };

    let command_name_str = command_name.value();

    let comp_entry = get_comp_entry(&pack);

    let dispatch_tree_entry = get_dispatch_tree_entry(&command_name_str, &command_struct, &pack);

    let expanded = {
        let program_path = group_path;

        quote! {
            #[derive(Debug, Default)]
            pub struct #command_struct;

            ::mingling::macros::pack!(#program_path, #pack = Vec<String>);

            #comp_entry
            #dispatch_tree_entry

            impl ::mingling::Dispatcher<#program_path> for #command_struct {
                fn node(&self) -> ::mingling::Node {
                    ::mingling::macros::node!(#command_name_str)
                }
                fn begin(&self, args: Vec<String>) -> ::mingling::ChainProcess<#program_path> {
                    #pack::new(args).to_chain()
                }
                fn clone_dispatcher(&self) -> Box<dyn ::mingling::Dispatcher<#program_path>> {
                    Box::new(#command_struct)
                }
            }
        }
    };

    expanded.into()
}

#[cfg(feature = "comp")]
fn get_comp_entry(entry_name: &Ident) -> TokenStream2 {
    let comp_entry = quote! {
        impl ::mingling::CompletionEntry for #entry_name {
            fn get_input(self) -> Vec<String> {
                self.inner.clone()
            }
        }
    };
    comp_entry
}

#[cfg(not(feature = "comp"))]
fn get_comp_entry(_entry_name: &Ident) -> TokenStream2 {
    quote! {}
}

#[cfg(feature = "dispatch_tree")]
fn get_dispatch_tree_entry(
    command_name_str: &str,
    command_struct: &Ident,
    entry_name: &Ident,
) -> TokenStream2 {
    let node_name_lit = syn::LitStr::new(command_name_str, proc_macro2::Span::call_site());
    quote! {
        ::mingling::macros::register_dispatcher!(#node_name_lit, #command_struct, #entry_name);
    }
}

#[cfg(not(feature = "dispatch_tree"))]
fn get_dispatch_tree_entry(
    _command_name_str: &str,
    _command_struct: &Ident,
    _entry_name: &Ident,
) -> TokenStream2 {
    quote! {}
}

#[cfg(feature = "dispatch_tree")]
/// Input format: ("node.name", DispatcherType, EntryName)
struct RegisterDispatcherInput {
    node_name: syn::LitStr,
    dispatcher_type: Ident,
    entry_name: Ident,
}

#[cfg(feature = "dispatch_tree")]
impl Parse for RegisterDispatcherInput {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let node_name = input.parse()?;
        input.parse::<Token![,]>()?;
        let dispatcher_type = input.parse()?;
        input.parse::<Token![,]>()?;
        let entry_name = input.parse()?;
        Ok(RegisterDispatcherInput {
            node_name,
            dispatcher_type,
            entry_name,
        })
    }
}

#[cfg(feature = "dispatch_tree")]
pub fn register_dispatcher(input: TokenStream) -> TokenStream {
    let RegisterDispatcherInput {
        node_name,
        dispatcher_type,
        entry_name,
    } = syn::parse_macro_input!(input as RegisterDispatcherInput);

    let node_name_str = node_name.value();
    let static_name = format!("__internal_dispatcher_{}", node_name_str.replace('.', "_"));
    let static_ident = Ident::new(&static_name, proc_macro2::Span::call_site());

    // Register node info in the global collection at compile time
    // Format: "node.name:DispatcherType:EntryName"
    COMPILE_TIME_DISPATCHERS.lock().unwrap().insert(format!(
        "{}:{}:{}",
        node_name_str, dispatcher_type, entry_name
    ));

    let expanded = quote! {
        #[doc(hidden)]
        #[allow(nonstandard_style)]
        pub static #static_ident: #dispatcher_type = #dispatcher_type;
    };

    expanded.into()
}

#[cfg(not(feature = "dispatch_tree"))]
pub fn register_dispatcher(_input: TokenStream) -> TokenStream {
    quote! {}.into()
}
