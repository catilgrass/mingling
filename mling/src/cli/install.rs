use mingling::{
    ShellContext, Suggest,
    macros::{chain, completion, dispatcher, pack, suggest},
    parser::Picker,
};

use crate::project_installer::install_all;

dispatcher!("install", InstallCommand => InstallEntry);

pack!(ResultInstallCompleted = ());

#[completion(InstallEntry)]
pub(crate) fn comp_install(ctx: &ShellContext) -> Suggest {
    if ctx.typing_argument() {
        return suggest! {
            "--clean": "Clean build artifacts before installation",
            "-c": "Clean build artifacts before installation",
        };
    }
    return suggest!();
}

#[chain]
pub(crate) fn handle_install_entry(prev: InstallEntry) -> Next {
    let is_clean_before_build = Picker::new(prev.inner)
        .pick::<bool>(["--clean", "-c"])
        .unpack();
    let _ = install_all(is_clean_before_build);

    ResultInstallCompleted::new(())
}
