use super::NiksiConfig;

use serde::Serialize;

/// The Schema based on which the devcontainer.json file is generated.
/// For a complete documentation see: https://containers.dev/implementors/json_reference/
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DevContainer {
    /// The name for the Dev Container
    name: String,
    /// The docker image that will be used to create the container.
    image: String,
    /// The username of the non-root user
    remote_user: &'static str,
    /// The customizations applied to the container
    customizations: Customizations,
}

/// VSCode specific tool configuration
#[derive(Serialize)]
pub struct VSCode {
    /// List of extensions available in the Dev Container.
    /// Extensions should be specified by their extension id (e.g. "scalameta.metals")
    extensions: Vec<String>,
}

/// Tool specific customizations for the Dev Container schema.
/// For now, we only implement support for specifying VSCode extensions
/// but this will be extended in the future.
#[non_exhaustive]
#[derive(Serialize)]
pub struct Customizations {
    vscode: VSCode,
}

impl From<NiksiConfig> for DevContainer {
    fn from(config: NiksiConfig) -> Self {
        Self {
            image: format!(
                "{}{}:{}",
                config.registry.map(|r| r + "/").unwrap_or_default(),
                &config.name,
                &config.version
            ),
            name: config.name
                + &config
                    .course_code
                    .map(|c| format!(" ({c})"))
                    .unwrap_or_default(),
            remote_user: "vscode",
            customizations: Customizations {
                vscode: VSCode {
                    extensions: config.vscode_extensions,
                },
            },
        }
    }
}
