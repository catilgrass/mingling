use crate::{
    GeneralRendererSetting, RenderResult, renderer::general::error::GeneralRendererSerializeError,
};
use serde::Serialize;

pub mod error;

/// A general renderer that supports multiple serialization formats.
///
/// The `GeneralRenderer` provides methods to serialize data into various formats
/// including JSON, YAML, TOML, and RON, with support for both regular and
/// pretty-printed variants. It is designed to work with types that implement
/// the `Serialize` trait.
pub struct GeneralRenderer;

impl GeneralRenderer {
    // Renders data in the specified format to the given RenderResult.
    #[allow(unused_variables)]
    pub fn render<T: Serialize + Send>(
        data: &T,
        setting: &GeneralRendererSetting,
        r: &mut RenderResult,
    ) -> Result<(), GeneralRendererSerializeError> {
        match setting {
            GeneralRendererSetting::Disable => Ok(()),
            #[cfg(feature = "json_serde_fmt")]
            GeneralRendererSetting::Json => Self::render_to_json(data, r),
            #[cfg(feature = "json_serde_fmt")]
            GeneralRendererSetting::JsonPretty => Self::render_to_json_pretty(data, r),
            #[cfg(feature = "yaml_serde_fmt")]
            GeneralRendererSetting::Yaml => Self::render_to_yaml(data, r),
            #[cfg(feature = "toml_serde_fmt")]
            GeneralRendererSetting::Toml => Self::render_to_toml(data, r),
            #[cfg(feature = "ron_serde_fmt")]
            GeneralRendererSetting::Ron => Self::render_to_ron(data, r),
            #[cfg(feature = "ron_serde_fmt")]
            GeneralRendererSetting::RonPretty => Self::render_to_ron_pretty(data, r),
        }
    }

    /// Serializes data to JSON format and writes it to the render result.
    #[cfg(feature = "json_serde_fmt")]
    pub fn render_to_json<T: Serialize + Send>(
        data: &T,
        r: &mut RenderResult,
    ) -> Result<(), GeneralRendererSerializeError> {
        let json_string = serde_json::to_string(data)
            .map_err(|e| GeneralRendererSerializeError::new(e.to_string()))?;
        r.print(json_string.to_string().as_str());
        Ok(())
    }

    /// Serializes data to pretty-printed JSON format and writes it to the render result.
    #[cfg(feature = "json_serde_fmt")]
    pub fn render_to_json_pretty<T: Serialize + Send>(
        data: &T,
        r: &mut RenderResult,
    ) -> Result<(), GeneralRendererSerializeError> {
        let json_string = serde_json::to_string_pretty(data)
            .map_err(|e| GeneralRendererSerializeError::new(e.to_string()))?;
        r.print(json_string.to_string().as_str());
        Ok(())
    }

    /// Serializes data to RON format and writes it to the render result.
    #[cfg(feature = "ron_serde_fmt")]
    pub fn render_to_ron<T: Serialize + Send>(
        data: &T,
        r: &mut RenderResult,
    ) -> Result<(), GeneralRendererSerializeError> {
        let ron_string = ron::ser::to_string(data)
            .map_err(|e| GeneralRendererSerializeError::new(e.to_string()))?;
        r.print(ron_string.to_string().as_str());
        Ok(())
    }

    /// Serializes data to pretty-printed RON format and writes it to the render result.
    #[cfg(feature = "ron_serde_fmt")]
    pub fn render_to_ron_pretty<T: Serialize + Send>(
        data: &T,
        r: &mut RenderResult,
    ) -> Result<(), GeneralRendererSerializeError> {
        let mut pretty_config = ron::ser::PrettyConfig::new();
        pretty_config.new_line = std::borrow::Cow::from("\n");
        pretty_config.indentor = std::borrow::Cow::from("  ");

        let ron_string = ron::ser::to_string_pretty(data, pretty_config)
            .map_err(|e| GeneralRendererSerializeError::new(e.to_string()))?;
        r.print(ron_string.to_string().as_str());
        Ok(())
    }

    /// Serializes data to TOML format and writes it to the render result.
    #[cfg(feature = "toml_serde_fmt")]
    pub fn render_to_toml<T: Serialize + Send>(
        data: &T,
        r: &mut RenderResult,
    ) -> Result<(), GeneralRendererSerializeError> {
        let toml_string =
            toml::to_string(data).map_err(|e| GeneralRendererSerializeError::new(e.to_string()))?;
        r.print(toml_string.to_string().as_str());
        Ok(())
    }

    /// Serializes data to YAML format and writes it to the render result.
    #[cfg(feature = "yaml_serde_fmt")]
    pub fn render_to_yaml<T: Serialize + Send>(
        data: &T,
        r: &mut RenderResult,
    ) -> Result<(), GeneralRendererSerializeError> {
        let yaml_string = serde_yaml::to_string(data)
            .map_err(|e| GeneralRendererSerializeError::new(e.to_string()))?;
        r.print(yaml_string.to_string().as_str());
        Ok(())
    }
}
