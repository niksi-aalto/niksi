mod devcontainer;

use anyhow::{Context, Result};

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
    /// The name of the produced docker image
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

/// The Builder helper for the [`Niksi`] struct
///
/// Example
/// ```rust
/// let niksi = Niksi::builder()
///     .config_file("niksi.json")
///     .output_directory("./outdir")
///     .lock_file("./outdir/niksi.lock")
///     .build()?;
///
/// niksi.build();
/// ```
#[derive(Default, Debug, Clone)]
pub struct NiksiBuilder {
    config_file: Option<PathBuf>,
    output_directory: Option<PathBuf>,
    lock_file: Option<PathBuf>,
}

impl Niksi {
    /// Constructs a new [`NiksiBuilder`].
    /// See the builders documentation for more information.
    pub fn builder() -> NiksiBuilder {
        NiksiBuilder::new()
    }

    /// Outputs the JSON for .devcontainer.json as a String.
    /// Fails if Serialization fails.
    pub fn devcontainer_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&DevContainer::from(self.config.clone()))
    }

    /// Outputs the overrides.nix file with paths, name and version set according to the
    /// [`NiksiConfig`] provided.
    pub fn overrides_nix(&self) -> String {
        format!(
            r#"{{pkgs, ...}}: {{
                paths = with pkgs; [{}];
                name = "{}";
                tag = "{}";
            }}"#,
            self.config.packages.join(" "),
            &self.config.name,
            &self.config.version,
        )
    }

    /// Builds the docker image as specified in the [`NiksiConfig`] by:
    /// 1. Downloading the template or a default template
    /// 2. Writing the result of [`Niksi::overrides_nix`] to overrides.nix
    /// 3. Copying the niksi.lock file over, if one exists
    /// 4. Running `nix build`
    /// 5. Returning the path of the build result (in the Nix store)
    ///
    /// Fails if any of the Nix commands or fs access fails
    pub fn build(&self) -> Result<PathBuf> {
        const DEFAULT_TEMPLATE: &str = "github:niksi-aalto/templates#plain";

        let workdir = tempfile::tempdir().context("Failed to create tempdir to work in")?;

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
            .status()
            .context("`nix flake init` failed with the specified template")?;

        let overrides = self.overrides_nix();
        let mut overrides_file = std::fs::File::create(workdir.path().join("overrides.nix"))
            .context("Failed to create overrides.nix file")?;
        overrides_file
            .write_all(overrides.as_bytes())
            .context("Failed to write overrides to overrides.nix")?;

        if self.lock_file.is_file() {
            std::fs::copy(self.lock_file.as_path(), workdir.path().join("flake.lock"))
                .context("Failed to copy niksi lockfile")?;
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
            .output()
            .context("Nix build failed")?;

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
    /// Constructs a new [`NiksiBuilder`] similarly to [`Niksi::builder`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Specify the niksi json config file.
    /// For the schema of the config file see [`NiksiConfig`]
    ///
    /// The config file is *required*, meaning that [`Niksibuilder::build`] will fail if no config
    /// file has been specified.
    pub fn config_file(mut self, file: impl Into<PathBuf>) -> Self {
        self.config_file = Some(file.into());
        self
    }

    /// Specify the output directory, if no directory is specified, the current directory is used.
    pub fn output_directory(mut self, dir: impl Into<PathBuf>) -> Self {
        self.output_directory = Some(dir.into());
        self
    }

    /// Specify the lock file. If no file is specified a new lock file is generated upon running [`Niksi::build`].
    pub fn lock_file(mut self, file: impl Into<PathBuf>) -> Self {
        self.lock_file = Some(file.into());
        self
    }

    /// Construct a new [`Niksi`] struct.
    ///
    /// fails if config file specification is invalid or missing.
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
