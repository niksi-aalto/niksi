mod devcontainer;

use serde::Deserialize;
use std::path::PathBuf;
use thiserror::Error;

/// The Schema of the niksi.json configuration file used for configuring a Dev Container
#[derive(Deserialize, Debug, Clone)]
pub struct NiksiConfig {
    /// The name of the course or project
    pub name: String,
    /// Optional course code
    pub course_code: Option<String>,
    /// The version of the course Dev Container
    pub version: String,
    /// The maintainer(s) of the course Dev Container
    pub maintainers: Vec<String>,
    /// Additional packages added to the Dev Container
    pub packages: Vec<String>,
    /// Extensions installed to the VSCode running in the Dev Container
    pub vscode_extensions: Vec<String>,
    /// The Niksi-template used
    pub template: Option<String>,
    /// The Docker registry used for the container image
    pub registry: Option<String>,
}

#[derive(Debug, Clone)]
struct Niksi {
    config: NiksiConfig,
    working_directory: PathBuf,
}

#[derive(Default, Debug, Clone)]
struct NiksiBuilder {
    config_file: Option<PathBuf>,
    working_directory: Option<PathBuf>,
}

impl Niksi {
    pub fn builder() -> NiksiBuilder {
        NiksiBuilder::new()
    }
}

#[derive(Error, Debug)]
enum BuilderError {
    #[error("Failed to read configuration file {0}")]
    NoSuchFile(#[from] std::io::Error),
    #[error("Malformed configuration file: {0}")]
    ParseError(#[from] serde_json::Error),
    #[error("Incomplete build, missing required field {0}")]
    Incomplete(String),
}

impl NiksiBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn config_file(mut self, file: impl Into<PathBuf>) -> Self {
        self.config_file = Some(file.into());
        self
    }

    pub fn working_directory(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_directory = Some(dir.into());
        self
    }

    pub fn build(self) -> Result<Niksi, BuilderError> {
        let config = match self.config_file {
            Some(p) => {
                let file = std::fs::File::open(p)?;
                let config: NiksiConfig = serde_json::from_reader(file)?;
                config
            }
            _ => return Err(BuilderError::Incomplete("config_file".into())),
        };

        Ok(Niksi {
            config,
            working_directory: self.working_directory.unwrap_or_default(),
        })
    }
}
