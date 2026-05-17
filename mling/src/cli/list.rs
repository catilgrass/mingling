use colored::Colorize;
use mingling::{
    Groupped, RenderResult, ShellContext, Suggest,
    macros::{chain, completion, dispatcher, pack, r_println, renderer, suggest},
    parser::Picker,
};
use serde::Serialize;

use crate::namespace_manager::list_namespaces;

dispatcher!("ls-namespace", ListInstalledCommand => ListInstalledEntry);

#[completion(ListInstalledEntry)]
pub(crate) fn comp_list_installed(ctx: &ShellContext) -> Suggest {
    if ctx.typing_argument() {
        return suggest! {
            "--trusted": "Show only trusted namespaces",
            "--untrusted": "Show only untrusted namespaces",
        };
    }
    return suggest!();
}

#[derive(Debug, Serialize, Default, Groupped)]
pub(crate) enum StateListInstalledOptions {
    #[default]
    All,
    OnlyTrusted,
    OnlyUntrusted,
}

pack!(MutexErrorListInstalled = ());

#[chain]
pub(crate) fn handle_list_installed_entry(prev: ListInstalledEntry) -> Next {
    let picker = Picker::new(prev.inner);
    let r = picker
        .pick::<bool>("--trusted")
        .pick::<bool>("--untrusted")
        .unpack();

    let option: StateListInstalledOptions = match r {
        // (show_trusted, show_untrusted)
        (true, false) => StateListInstalledOptions::OnlyTrusted,
        (false, true) => StateListInstalledOptions::OnlyUntrusted,
        (false, false) => StateListInstalledOptions::All,
        (true, true) => return MutexErrorListInstalled::default().to_render(),
    };

    option.to_chain()
}

#[renderer]
pub(crate) fn render_list_installed_mutex_error(_prev: MutexErrorListInstalled) {
    r_println!("Error: cannot use both --trusted and --untrusted options at the same time")
}

#[derive(Debug, Groupped, Serialize)]
pub(crate) struct ResultInstalledNamespaces {
    trusted: Vec<String>,
    untrusted: Vec<String>,
    untagged: Vec<String>,
    option: StateListInstalledOptions,
}

#[chain]
pub(crate) fn handle_state_list_installed_option(prev: StateListInstalledOptions) -> Next {
    ResultInstalledNamespaces {
        trusted: list_namespaces(true, false, false),
        untrusted: list_namespaces(false, true, false),
        untagged: list_namespaces(false, false, true),
        option: prev,
    }
}

#[renderer]
pub(crate) fn render_installed(prev: ResultInstalledNamespaces) {
    match prev.option {
        StateListInstalledOptions::All => {
            print_list("Trusted".bright_green().bold().to_string(), prev.trusted, r);
            print_list(
                "Untrusted".bright_red().bold().to_string(),
                prev.untrusted,
                r,
            );
            print_list(
                "Untagged".bright_black().bold().to_string(),
                prev.untagged,
                r,
            );
        }
        StateListInstalledOptions::OnlyTrusted => {
            print_list("Trusted".bright_green().bold().to_string(), prev.trusted, r);
        }
        StateListInstalledOptions::OnlyUntrusted => {
            print_list(
                "Untrusted".bright_red().bold().to_string(),
                prev.untrusted,
                r,
            );
        }
    }
}

fn print_list(title: String, list: Vec<String>, r: &mut RenderResult) {
    if list.is_empty() {
        return;
    }

    r_println!("{}", title);

    for (i, namespace) in (1..).zip(list.iter()) {
        r_println!("  {}. {}", i.to_string(), namespace.bold());
    }
}
