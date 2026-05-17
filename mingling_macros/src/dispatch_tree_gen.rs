use proc_macro2::TokenStream;
use quote::quote;
use std::collections::BTreeMap;

/// Generate the `get_nodes()` function body for a ProgramCollect impl.
pub fn gen_get_nodes(entries: &[(String, String, String)]) -> TokenStream {
    let mut node_entries = Vec::new();

    for (node_name, _disp_type, _entry_name) in entries {
        let static_name_str = format!("__internal_dispatcher_{}", node_name.replace('.', "_"));
        let static_ident = syn::Ident::new(&static_name_str, proc_macro2::Span::call_site());

        let node_display_name = node_name.replace('.', " ");
        let node_display_lit = syn::LitStr::new(&node_display_name, proc_macro2::Span::call_site());

        node_entries.push(quote! {
            (#node_display_lit.to_string(), & #static_ident)
        });
    }

    quote! {
        fn get_nodes() -> Vec<(String, &'static (dyn ::mingling::Dispatcher<Self::Enum> + Send + Sync))> {
            vec![
                #(#node_entries),*
            ]
        }
    }
}

/// Generate the `dispatch_args_trie()` function body for a ProgramCollect impl.
///
/// Builds a hardcoded match tree: at each depth, group nodes by character.
/// Single-node groups use `starts_with`; multi-node groups recurse with `nth()` match.
pub fn gen_dispatch_args_trie(entries: &[(String, String, String)]) -> TokenStream {
    // Prepare (display_name, disp_type) pairs.
    // display_name = node_name.replace('.', " ")
    let nodes: Vec<(String, String)> = entries
        .iter()
        .map(|(name, disp, _)| (name.replace('.', " "), disp.clone()))
        .collect();

    let dispatch_body = build_dispatch_body(&nodes, 0);

    quote! {
        fn dispatch_args_trie(
            raw: &Vec<String>,
        ) -> Result<::mingling::AnyOutput<Self::Enum>, ::mingling::error::ProgramInternalExecuteError>
        {
            let raw_string = format!("{} ", raw.join(" "));
            let raw_str = raw_string.as_str();
            let mut raw_chars = raw_str.chars();
            #dispatch_body
        }
    }
}

/// Recursively build the trie match body.
///
/// `nodes`: slice of (display_name, disp_type) for commands that share the same prefix so far.
/// `depth`: The character index currently being matched.
///
/// Returns a `TokenStream` representing the match block at this depth.
fn build_dispatch_body(nodes: &[(String, String)], depth: usize) -> TokenStream {
    if nodes.is_empty() {
        return quote! {
            return Ok(Self::build_dispatcher_not_found(raw.clone()));
        };
    }

    // Group by character at `depth`
    // Nodes that end exactly at this depth (name is closed – rare but possible, e.g. "hell")
    // are collected into `exact_nodes`. All others go into `groups[char]`.
    let mut groups: BTreeMap<char, Vec<(String, String)>> = BTreeMap::new();
    let mut exact_nodes: Vec<(String, String)> = Vec::new();

    for (name, disp_type) in nodes {
        if let Some(ch) = name.chars().nth(depth) {
            groups
                .entry(ch)
                .or_default()
                .push((name.clone(), disp_type.clone()));
        } else {
            exact_nodes.push((name.clone(), disp_type.clone()));
        }
    }

    // Build a dispatch arm for a single node via `starts_with`
    let make_starts_with_arm = |name: &str, disp_type: &str| -> TokenStream {
        let name_space = format!("{} ", name);
        let name_lit = syn::LitStr::new(&name_space, proc_macro2::Span::call_site());
        let disp_ident = syn::Ident::new(disp_type, proc_macro2::Span::call_site());
        let prefix_word_count = name.split_whitespace().count();
        quote! {
            if raw_str.starts_with(#name_lit) {
                let prefix_len = #prefix_word_count;
                let trimmed_args: Vec<String> = raw.iter().skip(prefix_len).cloned().collect();
                let __cp = <#disp_ident as ::mingling::Dispatcher<Self::Enum>>::begin(
                    &#disp_ident::default(),
                    trimmed_args,
                );
                return match __cp {
                    ::mingling::ChainProcess::Ok(any_output) => Ok(any_output.0),
                    ::mingling::ChainProcess::Err(chain_process_error) => {
                        Err(chain_process_error.into())
                    }
                };
            }
        }
    };

    // Build match arms
    let mut arms = Vec::new();

    for (&ch, sub_nodes) in &groups {
        let ch_char = ch;

        if sub_nodes.len() == 1 {
            // Only one candidate – use `starts_with` directly at this depth.
            let (name, disp_type) = &sub_nodes[0];
            let arm = make_starts_with_arm(name, disp_type);
            arms.push(quote! {
                Some(#ch_char) => {
                    #arm
                    return Ok(Self::build_dispatcher_not_found(raw.clone()));
                }
            });
        } else {
            // Multiple candidates – recurse deeper
            let sub_body = build_dispatch_body(sub_nodes, depth + 1);
            arms.push(quote! {
                Some(#ch_char) => {
                    #sub_body
                }
            });
        }
    }

    // Exact-match nodes at this depth
    // These are names that are a prefix of other names (e.g. "hell" when "hello" exists).
    // They are tried first via `starts_with`, then fall through to `raw_chars.nth(depth)` match.
    let exact_checks: Vec<TokenStream> = exact_nodes
        .iter()
        .map(|(name, disp_type)| make_starts_with_arm(name, disp_type))
        .collect();

    // Assemble
    // If there are exact nodes, first check starts_with, then do matching.
    if !exact_checks.is_empty() && !groups.is_empty() {
        // Exact nodes + deeper groups: do starts_with checks, then match on nth(depth)
        let match_body = quote! {
            match raw_chars.nth(0) {
                #(#arms)*
                _ => return Ok(Self::build_dispatcher_not_found(raw.clone())),
            }
        };
        quote! {
            #(#exact_checks)*
            #match_body
        }
    } else if !exact_checks.is_empty() {
        // Only exact nodes, no deeper groups
        quote! {
            #(#exact_checks)*
            return Ok(Self::build_dispatcher_not_found(raw.clone()));
        }
    } else if arms.is_empty() {
        // Only fallback (shouldn't happen if nodes is non-empty)
        quote! {
            return Ok(Self::build_dispatcher_not_found(raw.clone()));
        }
    } else {
        // Only group arms
        quote! {
            match raw_chars.nth(0) {
                #(#arms)*
                _ => return Ok(Self::build_dispatcher_not_found(raw.clone())),
            }
        }
    }
}
