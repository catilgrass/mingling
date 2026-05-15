use std::path::PathBuf;

use just_template::tmpl_param;

use crate::ShellFlag;

const TMPL_COMP_BASH: &str = include_str!("../../tmpls/comps/bash.sh");
const TMPL_COMP_ZSH: &str = include_str!("../../tmpls/comps/zsh.zsh");
const TMPL_COMP_FISH: &str = include_str!("../../tmpls/comps/fish.fish");
const TMPL_COMP_PWSH: &str = include_str!("../../tmpls/comps/pwsh.ps1");

/// Generate shell completion scripts for a given binary name.
/// On Windows, generates PowerShell completion.
/// On Linux, generates Zsh, Bash, and Fish completions.
/// Scripts are written to the `OUT_DIR` (or `target/` if `OUT_DIR` is not set).
///
/// # Example
/// ```rust,ignore
/// # use mingling_core::comp::ShellFlag;
/// # use mingling_core::build::build_comp_scripts;
///
/// // Generate completion scripts for "myapp"
/// build_comp_scripts("myapp").unwrap();
///
/// // Generate completion scripts for current package
/// build_comp_scripts(env!("CARGO_PKG_NAME")).unwrap();
/// ```
pub fn build_comp_scripts(name: &str) -> Result<(), std::io::Error> {
    #[cfg(target_os = "windows")]
    {
        build_comp_script(&ShellFlag::Powershell, name)?;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        build_comp_script(&ShellFlag::Zsh, name)?;
        build_comp_script(&ShellFlag::Bash, name)?;
        build_comp_script(&ShellFlag::Fish, name)?;
        Ok(())
    }

    #[cfg(target_os = "macos")]
    {
        build_comp_script(&ShellFlag::Zsh, name)?;
        build_comp_script(&ShellFlag::Bash, name)?;
        build_comp_script(&ShellFlag::Fish, name)?;
        Ok(())
    }
}

/// Generate a shell completion script for a specific shell.
///
/// This function takes a shell flag and a binary name, selects the appropriate
/// template, substitutes the binary name into the template, and writes the
/// resulting completion script to the target directory (typically `target/`).
///
/// # Example
/// ```rust,ignore
/// # use mingling_core::comp::ShellFlag;
/// # use mingling_core::build::build_comp_script;
/// build_comp_script(&ShellFlag::Bash, "myapp").unwrap();
/// ```
pub fn build_comp_script(shell_flag: &ShellFlag, bin_name: &str) -> Result<(), std::io::Error> {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let target_dir = out_dir.join("../../../").to_path_buf();
    build_comp_script_to(shell_flag, bin_name, &target_dir.to_string_lossy())
}

/// Generate a shell completion script to a specified directory.
///
/// This function takes a shell flag, a binary name, and a target directory path,
/// selects the appropriate template, substitutes the binary name into the template,
/// and writes the resulting completion script to the specified directory.
///
/// # Example
/// ```rust,ignore
/// # use mingling_core::comp::ShellFlag;
/// # use mingling_core::build::build_comp_script_to;
/// build_comp_script_to(&ShellFlag::Bash, "myapp", "target/completions").unwrap();
/// ```
pub fn build_comp_script_to(
    shell_flag: &ShellFlag,
    bin_name: &str,
    target_dir: &str,
) -> Result<(), std::io::Error> {
    let (tmpl_str, ext) = get_tmpl(shell_flag);
    let mut tmpl = just_template::Template::from(tmpl_str);
    tmpl_param!(tmpl, bin_name = bin_name);
    let target_path = std::path::PathBuf::from(target_dir);
    std::fs::create_dir_all(&target_path)?;
    let output_path = target_path.join(format!("{}_comp{}", bin_name, ext));
    std::fs::write(&output_path, tmpl.to_string())
}

/// Generate a shell completion script and write it to a specified file path.
///
/// This function takes a shell flag, a binary name, and an output file path,
/// selects the appropriate template, substitutes the binary name into the template,
/// and writes the resulting completion script directly to the specified file path.
///
/// # Example
/// ```rust,ignore
/// # use mingling_core::comp::ShellFlag;
/// # use mingling_core::build::build_comp_script_to_file;
/// build_comp_script_to_file(&ShellFlag::Bash, "myapp", "target/completions/myapp_comp.sh").unwrap();
/// ```
pub fn build_comp_script_to_file(
    shell_flag: &ShellFlag,
    bin_name: &str,
    output_path: impl Into<PathBuf>,
) -> Result<(), std::io::Error> {
    let (tmpl_str, _ext) = get_tmpl(shell_flag);
    let mut tmpl = just_template::Template::from(tmpl_str);
    tmpl_param!(tmpl, bin_name = bin_name);
    std::fs::write(output_path.into(), tmpl.to_string())
}

fn get_tmpl(shell_flag: &ShellFlag) -> (&'static str, &'static str) {
    match shell_flag {
        ShellFlag::Bash => (TMPL_COMP_BASH, ".sh"),
        ShellFlag::Zsh => (TMPL_COMP_ZSH, ".zsh"),
        ShellFlag::Fish => (TMPL_COMP_FISH, ".fish"),
        ShellFlag::Powershell => (TMPL_COMP_PWSH, ".ps1"),
        ShellFlag::Other(_) => (TMPL_COMP_BASH, ".sh"),
    }
}
