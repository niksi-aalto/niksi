mod devcontainer;

use serde::Deserialize;

/// The Schema of the niksi.json configuration file used for configuring a Dev Container
#[derive(Deserialize)]
struct NiksiConfig {
    /// The name of the course or project
    name: String,
    /// Optional course code
    course_code: Option<String>,
    /// The version of the course Dev Container
    version: String,
    /// The maintainer(s) of the course Dev Container
    maintainers: Vec<String>,
    /// Additional packages added to the Dev Container
    packages: Vec<String>,
    /// Extensions installed to the VSCode running in the Dev Container
    vscode_extensions: Vec<String>,
    /// The Niksi-template used
    template: Option<String>,
    /// The Docker registry used for the container image
    registry: Option<String>,
}
