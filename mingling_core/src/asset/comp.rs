mod flags;
mod shell_ctx;
mod suggest;

use std::collections::BTreeSet;
use std::fmt::Display;

#[doc(hidden)]
pub use flags::*;
#[doc(hidden)]
pub use shell_ctx::*;
#[doc(hidden)]
pub use suggest::*;

use crate::{ProgramCollect, debug, exec::match_user_input, only_debug, this, trace};

/// Trait for implementing completion logic.
///
/// This trait defines the interface for generating command-line completions.
/// Types implementing this trait can provide custom completion suggestions
/// based on the current shell context.
pub trait Completion {
    type Previous;
    fn comp(ctx: &ShellContext) -> Suggest;
}

/// Trait for extracting user input arguments for completion.
///
/// When the `feat comp` feature is enabled, the `dispatcher!` macro will
/// automatically implement this trait for `Entry` types to extract the
/// arguments from user input for completion suggestions.
pub trait CompletionEntry {
    fn get_input(self) -> Vec<String>;
}

/// A helper struct for handling command-line completion logic.
///
/// This struct provides static methods for executing completions based on
/// the current shell context and rendering the resulting suggestions in a
/// format appropriate for the target shell.
pub struct CompletionHelper;
impl CompletionHelper {
    pub fn exec_completion<P>(ctx: &ShellContext) -> Suggest
    where
        P: ProgramCollect<Enum = P> + Display + 'static,
    {
        only_debug! {
            crate::debug::init_env_logger();
            trace_ctx(ctx);
        };

        let program = this::<P>();
        let args = ctx.all_words.iter().skip(1).cloned().collect::<Vec<_>>();
        let suggest = if let Ok((dispatcher, args)) = match_user_input(program, &args) {
            trace!(
                "dispatcher matched, dispatcher=\"{}\", args={:?}",
                dispatcher.node().to_string(),
                args
            );
            let begin = dispatcher.begin(args);
            if let crate::ChainProcess::Ok((any, _)) = begin {
                trace!("entry type: {}", any.member_id);
                let result = P::do_comp(&any, ctx);
                trace!("do_comp result: {:?}", result);
                Some(result)
            } else {
                trace!("begin not Ok");
                None
            }
        } else {
            trace!("no dispatcher matched");
            None
        };

        match suggest {
            Some(suggest) => {
                trace!("using custom completion: {:?}", suggest);
                suggest
            }
            None => {
                trace!("using default completion");
                default_completion::<P>(ctx)
            }
        }
    }

    pub fn render_suggest<P>(ctx: ShellContext, suggest: Suggest)
    where
        P: ProgramCollect<Enum = P> + Display + 'static,
    {
        trace!("render_suggest called with: {:?}", suggest);
        match suggest {
            Suggest::FileCompletion => {
                trace!("rendering file completion");
                println!("_file_");
                std::process::exit(0);
            }
            Suggest::Suggest(suggestions) => {
                trace!("rendering {} suggestions", suggestions.len());
                match ctx.shell_flag {
                    ShellFlag::Zsh | ShellFlag::Powershell => {
                        trace!("using zsh/pwsh format");
                        print_suggest_with_description(suggestions)
                    }
                    ShellFlag::Fish => {
                        trace!("using fish format");
                        print_suggest_with_description_fish(suggestions)
                    }
                    _ => {
                        trace!("using default format");
                        print_suggest(suggestions)
                    }
                }
            }
        }
    }
}

fn default_completion<P>(ctx: &ShellContext) -> Suggest
where
    P: ProgramCollect<Enum = P> + Display + 'static,
{
    let cmd_nodes: Vec<String> = this::<P>()
        .get_nodes()
        .into_iter()
        .filter(|(s, _)| s != "__comp")
        .map(|(s, _)| s)
        .collect();
    debug!("cmd_nodes: {:?}", cmd_nodes);

    // If the current position is less than 1, do not perform completion
    if ctx.word_index < 1 {
        debug!("word_index < 1, returning file suggestions");
        return file_suggest();
    };

    // Get the current input path
    debug!(
        "input_path before filter: {:?}",
        &ctx.all_words.get(1..ctx.word_index).unwrap_or(&[])
    );

    let input_path: Vec<&str> = ctx
        .all_words
        .get(1..ctx.word_index)
        .unwrap_or(&[])
        .iter()
        .filter(|s| !s.is_empty())
        .map(|s| s.as_str())
        .collect();
    debug!("input_path after filter: {:?}", input_path);

    debug!(
        "default_completion: input_path = {:?}, word_index = {}, all_words = {:?}",
        input_path, ctx.word_index, ctx.all_words
    );

    // Filter command nodes that match the input path
    let mut suggestions = Vec::new();

    // Special case: if input_path is empty, return all first-level commands
    if input_path.is_empty() {
        for node in cmd_nodes {
            let node_parts: Vec<&str> = node.split(' ').collect();
            if !node_parts.is_empty() && !suggestions.contains(&node_parts[0].to_string()) {
                suggestions.push(node_parts[0].to_string());
            }
        }
    } else {
        // Get the current word
        let current_word = input_path.last().unwrap();

        // First, handle partial match completion for the current word
        // Only perform current word completion when current_word is not empty
        if input_path.len() == 1 && !ctx.current_word.is_empty() {
            for node in &cmd_nodes {
                let node_parts: Vec<&str> = node.split(' ').collect();
                if !node_parts.is_empty()
                    && node_parts[0].starts_with(current_word)
                    && !suggestions.contains(&node_parts[0].to_string())
                {
                    suggestions.push(node_parts[0].to_string());
                }
            }

            // If suggestions for the current word are found, return directly
            if !suggestions.is_empty() {
                suggestions.sort();
                suggestions.dedup();
                debug!(
                    "default_completion: current word suggestions = {:?}",
                    suggestions
                );
                return suggestions.into();
            }
        }

        // Handle next-level command suggestions
        for node in cmd_nodes {
            let node_parts: Vec<&str> = node.split(' ').collect();

            debug!("Checking node: '{}', parts: {:?}", node, node_parts);

            // If input path is longer than node parts, skip
            if input_path.len() > node_parts.len() {
                continue;
            }

            // Check if input path matches the beginning of node parts
            let mut matches = true;
            for i in 0..input_path.len() {
                if i >= node_parts.len() {
                    matches = false;
                    break;
                }

                if i == input_path.len() - 1 {
                    if !node_parts[i].starts_with(input_path[i]) {
                        matches = false;
                        break;
                    }
                } else if input_path[i] != node_parts[i] {
                    matches = false;
                    break;
                }
            }

            if matches && input_path.len() <= node_parts.len() {
                if input_path.len() == node_parts.len() && !ctx.current_word.is_empty() {
                    suggestions.push(node_parts[input_path.len() - 1].to_string());
                } else if input_path.len() < node_parts.len() {
                    suggestions.push(node_parts[input_path.len()].to_string());
                }
            }
        }
    }

    // Remove duplicates and sort
    suggestions.sort();
    suggestions.dedup();

    debug!("default_completion: suggestions = {:?}", suggestions);

    if suggestions.is_empty() {
        file_suggest()
    } else {
        suggestions.into()
    }
}

fn file_suggest() -> Suggest {
    trace!("file_suggest called");
    Suggest::FileCompletion
}

fn print_suggest(suggestions: BTreeSet<SuggestItem>) {
    trace!("print_suggest called with {} items", suggestions.len());
    let mut sorted_suggestions: Vec<SuggestItem> = suggestions.into_iter().collect();
    sorted_suggestions.sort();

    for suggest in sorted_suggestions {
        println!("{}", suggest.suggest());
    }
    std::process::exit(0);
}

fn print_suggest_with_description(suggestions: BTreeSet<SuggestItem>) {
    trace!(
        "print_suggest_with_description called with {} items",
        suggestions.len()
    );
    let mut sorted_suggestions: Vec<SuggestItem> = suggestions.into_iter().collect();
    sorted_suggestions.sort();

    for suggest in sorted_suggestions {
        match suggest.description() {
            Some(desc) => println!("{}$({})", suggest.suggest(), desc),
            None => println!("{}", suggest.suggest()),
        }
    }
    std::process::exit(0);
}

fn print_suggest_with_description_fish(suggestions: BTreeSet<SuggestItem>) {
    trace!(
        "print_suggest_with_description_fish called with {} items",
        suggestions.len()
    );
    let mut sorted_suggestions: Vec<SuggestItem> = suggestions.into_iter().collect();
    sorted_suggestions.sort();

    for suggest in sorted_suggestions {
        match suggest.description() {
            Some(desc) => println!("{}\t{}", suggest.suggest(), desc),
            None => println!("{}", suggest.suggest()),
        }
    }
    std::process::exit(0);
}

#[cfg(feature = "debug")]
fn trace_ctx(ctx: &ShellContext) {
    trace!("=== SHELL CTX BEGIN ===");
    trace!("command_line={}", ctx.command_line);
    trace!("cursor_position={}", ctx.cursor_position);
    trace!("current_word={}", ctx.current_word);
    trace!("previous_word={}", ctx.previous_word);
    trace!("command_name={}", ctx.command_name);
    trace!("word_index={}", ctx.word_index);
    trace!("all_words={:?}", ctx.all_words);
    trace!("shell_flag={:?}", ctx.shell_flag);
    trace!("===  SHELL CTX END  ===");
}
