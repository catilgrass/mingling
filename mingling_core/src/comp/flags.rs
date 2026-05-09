use just_fmt::snake_case;

/// Represents the shell environment for which the output format is intended.
///
/// This enum defines the supported shell types that can be used for
/// generating shell-specific command syntax, scripts, or completions.
#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "general_renderer", derive(serde::Serialize))]
pub enum ShellFlag {
    /// Represents the Bash shell.
    #[default]
    Bash,
    /// Represents the Zsh shell.
    Zsh,
    /// Represents the Fish shell.
    Fish,
    /// Represents PowerShell.
    Powershell,
    /// A custom or unsupported shell type, identified by the provided string.
    Other(String),
}

impl From<String> for ShellFlag {
    fn from(s: String) -> Self {
        match s.trim().to_lowercase().as_str() {
            "zsh" => ShellFlag::Zsh,
            "bash" => ShellFlag::Bash,
            "fish" => ShellFlag::Fish,
            "pwsh" | "ps1" | "powershell" => ShellFlag::Powershell,
            other => ShellFlag::Other(snake_case!(other)),
        }
    }
}

impl From<ShellFlag> for String {
    fn from(flag: ShellFlag) -> Self {
        match flag {
            ShellFlag::Zsh => "zsh".to_string(),
            ShellFlag::Bash => "bash".to_string(),
            ShellFlag::Fish => "fish".to_string(),
            ShellFlag::Powershell => "powershell".to_string(),
            ShellFlag::Other(s) => s,
        }
    }
}
