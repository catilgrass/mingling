/// Program stdout settings
#[derive(Debug, Clone)]
pub struct ProgramStdoutSetting {
    /// Output error messages
    pub error_output: bool,

    /// Render results and output
    pub render_output: bool,

    #[cfg(feature = "clap")]
    /// Behavior when Clap Dispatcher outputs help information
    pub clap_help_print_behaviour: ClapHelpPrintBehaviour,
}

#[cfg(feature = "clap")]
#[derive(Debug, Default, Clone)]
pub enum ClapHelpPrintBehaviour {
    /// Write to RenderResult
    WriteToRenderResult,

    /// Print directly
    #[default]
    PrintDirectly,
}

impl Default for ProgramStdoutSetting {
    fn default() -> Self {
        ProgramStdoutSetting {
            error_output: true,
            render_output: true,
            #[cfg(feature = "clap")]
            clap_help_print_behaviour: ClapHelpPrintBehaviour::default(),
        }
    }
}

/// Program user context
#[derive(Debug, Clone)]
pub struct ProgramUserContext {
    /// View help information instead of running the command
    pub help: bool,

    /// Skip user confirmation step
    pub confirm: bool,

    /// Execute hooks during the program lifecycle
    pub run_hook: bool,
}

impl Default for ProgramUserContext {
    fn default() -> Self {
        Self {
            help: false,
            confirm: false,
            run_hook: true,
        }
    }
}

#[cfg(feature = "general_renderer")]
#[derive(Debug, Clone, Default)]
/// Settings for the general renderer output format.
///
/// Controls how structured data (e.g., JSON, YAML, TOML) is rendered to stdout.
pub enum GeneralRendererSetting {
    /// Do not render structured output (use default formatting).
    #[default]
    Disable,
    /// Render output as compact JSON.
    Json,
    /// Render output as pretty-printed JSON.
    JsonPretty,
    /// Render output as YAML.
    Yaml,
    /// Render output as TOML.
    Toml,
    /// Render output as RON.
    Ron,
    /// Render output as pretty-printed RON.
    RonPretty,
}

#[cfg(feature = "general_renderer")]
impl std::str::FromStr for GeneralRendererSetting {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match just_fmt::kebab_case!(s).as_str() {
            "disable" => Ok(GeneralRendererSetting::Disable),
            "json" => Ok(GeneralRendererSetting::Json),
            "json-pretty" => Ok(GeneralRendererSetting::JsonPretty),
            "yaml" => Ok(GeneralRendererSetting::Yaml),
            "toml" => Ok(GeneralRendererSetting::Toml),
            "ron" => Ok(GeneralRendererSetting::Ron),
            "ron-pretty" => Ok(GeneralRendererSetting::RonPretty),
            _ => Err(format!("Invalid renderer: '{}'", s)),
        }
    }
}

#[cfg(feature = "general_renderer")]
impl From<&str> for GeneralRendererSetting {
    fn from(s: &str) -> Self {
        s.parse().unwrap_or(GeneralRendererSetting::Disable)
    }
}

#[cfg(feature = "general_renderer")]
impl From<String> for GeneralRendererSetting {
    fn from(s: String) -> Self {
        s.as_str().into()
    }
}

#[cfg(feature = "general_renderer")]
impl std::fmt::Display for GeneralRendererSetting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GeneralRendererSetting::Disable => write!(f, "disable"),
            GeneralRendererSetting::Json => write!(f, "json"),
            GeneralRendererSetting::JsonPretty => write!(f, "json-pretty"),
            GeneralRendererSetting::Yaml => write!(f, "yaml"),
            GeneralRendererSetting::Toml => write!(f, "toml"),
            GeneralRendererSetting::Ron => write!(f, "ron"),
            GeneralRendererSetting::RonPretty => write!(f, "ron-pretty"),
        }
    }
}
