use mingling_core::{Program, ProgramCollect, setup::ProgramSetup};

/// Sets up the general renderer for the program:
///
/// - Adds a `--renderer` global argument to specify the renderer type
pub struct GeneralRendererSimpleSetup;

impl<C> ProgramSetup<C> for GeneralRendererSimpleSetup
where
    C: ProgramCollect<Enum = C>,
{
    fn setup(&mut self, program: &mut Program<C>) {
        program.global_argument("--renderer", |p, renderer| {
            p.general_renderer_name = renderer.into();
        });
    }
}

/// Sets up the general renderer for the program:
///
/// - Adds global flags to specify the renderer type:
///   * `--json` for JSON output
///   * `--json-pretty` for pretty-printed JSON output
///   * `--yaml` for YAML output
///   * `--toml` for TOML output
///   * `--ron` for RON output
///   * `--ron-pretty` for pretty-printed RON output
pub struct GeneralRendererSetup;

impl<C> ProgramSetup<C> for GeneralRendererSetup
where
    C: ProgramCollect<Enum = C>,
{
    #[allow(unused_variables)]
    fn setup(&mut self, program: &mut Program<C>) {
        #[cfg(feature = "json_serde_fmt")]
        program.global_flag("--json", |p| {
            p.general_renderer_name = crate::GeneralRendererSetting::Json
        });
        #[cfg(feature = "json_serde_fmt")]
        program.global_flag("--json-pretty", |p| {
            p.general_renderer_name = crate::GeneralRendererSetting::JsonPretty;
        });
        #[cfg(feature = "yaml_serde_fmt")]
        program.global_flag("--yaml", |p| {
            p.general_renderer_name = crate::GeneralRendererSetting::Yaml;
        });
        #[cfg(feature = "toml_serde_fmt")]
        program.global_flag("--toml", |p| {
            p.general_renderer_name = crate::GeneralRendererSetting::Toml;
        });
        #[cfg(feature = "ron_serde_fmt")]
        program.global_flag("--ron", |p| {
            p.general_renderer_name = crate::GeneralRendererSetting::Ron;
        });
        #[cfg(feature = "ron_serde_fmt")]
        program.global_flag("--ron-pretty", |p| {
            p.general_renderer_name = crate::GeneralRendererSetting::RonPretty;
        });
    }
}
