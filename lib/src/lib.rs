mod devcontainer;

use serde::Deserialize;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};
use thiserror::Error;

use std::io::Write;

use devcontainer::DevContainer;

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
pub struct Niksi {
    config: NiksiConfig,
    lock_file: PathBuf,
    output_directory: PathBuf,
}

#[derive(Default, Debug, Clone)]
pub struct NiksiBuilder {
    config_file: Option<PathBuf>,
    output_directory: Option<PathBuf>,
    lock_file: Option<PathBuf>,
}

impl Niksi {
    pub fn builder() -> NiksiBuilder {
        NiksiBuilder::new()
    }

    pub fn devcontainer_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&DevContainer::from(self.config.clone()))
    }

    pub fn packages_nix(&self) -> String {
        format!(
            "{{pkgs, ...}}:{{paths = with pkgs; [{}];}}",
            self.config.packages.join(" ")
        )
    }

    pub fn build(&self) -> Result<PathBuf, Box<dyn std::error::Error + 'static>> {
        const DEFAULT_TEMPLATE: &str = "github:niksi-aalto/templates#plain";

        let workdir = tempfile::tempdir()?;

        Command::new("nix")
            .args([
                "--extra-experimental-features",
                "nix-command flakes",
                "flake",
                "init",
                "-t",
            ])
            .arg(
                self.config
                    .template
                    .clone()
                    .map(|t| format!("github:niksi-aalto/templates#{}", t))
                    .unwrap_or(DEFAULT_TEMPLATE.to_string()),
            )
            .current_dir(workdir.path())
            .status()?;

        let extra_paths = format!(
            "{{pkgs, ...}}:with pkgs; [{}]",
            self.config.packages.join(" ")
        );
        let mut extras_file = std::fs::File::create(workdir.path().join("paths.nix"))?;
        extras_file.write_all(extra_paths.as_bytes())?;

        if self.lock_file.is_file() {
            std::fs::copy(self.lock_file.as_path(), workdir.path().join("flake.lock"))?;
        }

        let result = Command::new("nix")
            .args([
                "--extra-experimental-features",
                "nix-command flakes",
                "build",
                "--print-out-paths",
                ".",
            ])
            .current_dir(workdir.path())
            .stderr(Stdio::inherit())
            .output()?;

        std::fs::copy(
            workdir.path().join("flake.lock"),
            self.output_directory.as_path().join("niksi.lock"),
        )?;

        Ok(PathBuf::from(
            String::from_utf8(result.stdout).unwrap().trim(),
        ))
    }
}

#[derive(Error, Debug)]
pub enum BuilderError {
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

    pub fn output_directory(mut self, dir: impl Into<PathBuf>) -> Self {
        self.output_directory = Some(dir.into());
        self
    }

    pub fn lock_file(mut self, file: impl Into<PathBuf>) -> Self {
        self.lock_file = Some(file.into());
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
            lock_file: self.lock_file.unwrap_or_default(),
            output_directory: self.output_directory.unwrap_or_default(),
        })
    }
}
