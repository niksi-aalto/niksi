use super::NiksiConfig;

use serde::Serialize;

/// The Schema based on which the devcontainer.json file is generated.
/// For a complete documentation see: https://containers.dev/implementors/json_reference/
#[derive(Serialize)]
pub struct DevContainer {
    /// The name for the Dev Container
    name: String,
    /// The docker image that will be used to create the container.
    image: String,
    /// The customizations applied to the container
    customizations: Vec<Customizations>,
}

/// Tool specific customizations for the Dev Container schema.
/// For now, we only implement support for specifying VSCode extensions
/// but this will be extended in the future.
#[non_exhaustive]
#[derive(Serialize)]
enum Customizations {
    /// VSCode specific tool configuration
    #[serde(rename = "vscode")]
    VSCode {
        /// List of extensions available in the Dev Container.
        /// Extensions should be specified by their extension id (e.g. "scalameta.metals")
        extensions: Vec<String>,
    },
}

impl From<NiksiConfig> for DevContainer {
    fn from(config: NiksiConfig) -> Self {
        Self {
            image: format!(
                "{}{}:{}",
                config.registry.map(|r| r + "://").unwrap_or_default(),
                &config.name,
                &config.version
            ),
            name: config.name
                + &config
                    .course_code
                    .map(|c| format!(" ({c})"))
                    .unwrap_or_default(),
            customizations: vec![Customizations::VSCode {
                extensions: config.vscode_extensions,
            }],
        }
    }
}
