/// Program stdout settings
#[derive(Debug, Clone)]
pub struct ProgramStdoutSetting {
    /// Output error messages
    pub error_output: bool,

    /// Render results and output
    pub render_output: bool,

    /// Silence panic messages
    pub silence_panic: bool,

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
            silence_panic: false,
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
    #[cfg(feature = "json_serde_fmt")]
    Json,
    /// Render output as pretty-printed JSON.
    #[cfg(feature = "json_serde_fmt")]
    JsonPretty,
    /// Render output as YAML.
    #[cfg(feature = "yaml_serde_fmt")]
    Yaml,
    /// Render output as TOML.
    #[cfg(feature = "toml_serde_fmt")]
    Toml,
    /// Render output as RON.
    #[cfg(feature = "ron_serde_fmt")]
    Ron,
    /// Render output as pretty-printed RON.
    #[cfg(feature = "ron_serde_fmt")]
    RonPretty,
}

#[cfg(feature = "general_renderer")]
impl std::str::FromStr for GeneralRendererSetting {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match just_fmt::kebab_case!(s).as_str() {
            "disable" => Ok(GeneralRendererSetting::Disable),
            #[cfg(feature = "json_serde_fmt")]
            "json" => Ok(GeneralRendererSetting::Json),
            #[cfg(feature = "json_serde_fmt")]
            "json-pretty" => Ok(GeneralRendererSetting::JsonPretty),
            #[cfg(feature = "yaml_serde_fmt")]
            "yaml" => Ok(GeneralRendererSetting::Yaml),
            #[cfg(feature = "toml_serde_fmt")]
            "toml" => Ok(GeneralRendererSetting::Toml),
            #[cfg(feature = "ron_serde_fmt")]
            "ron" => Ok(GeneralRendererSetting::Ron),
            #[cfg(feature = "ron_serde_fmt")]
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
            #[cfg(feature = "json_serde_fmt")]
            GeneralRendererSetting::Json => write!(f, "json"),
            #[cfg(feature = "json_serde_fmt")]
            GeneralRendererSetting::JsonPretty => write!(f, "json-pretty"),
            #[cfg(feature = "yaml_serde_fmt")]
            GeneralRendererSetting::Yaml => write!(f, "yaml"),
            #[cfg(feature = "toml_serde_fmt")]
            GeneralRendererSetting::Toml => write!(f, "toml"),
            #[cfg(feature = "ron_serde_fmt")]
            GeneralRendererSetting::Ron => write!(f, "ron"),
            #[cfg(feature = "ron_serde_fmt")]
            GeneralRendererSetting::RonPretty => write!(f, "ron-pretty"),
        }
    }
}
