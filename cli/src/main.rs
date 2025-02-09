use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use lib::{BuilderError, Niksi};

use std::io::Write;

use tracing::{info, level_filters::LevelFilter};

extern crate tracing;

#[derive(Parser, Debug, Clone)]
#[command(version, about)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    Build {
        /// Whether to push the built image to the configured registry.
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        push: bool,
        /// Location of the config file.
        /// If no such file exists the program creates a sample configuration file in that
        /// location and then exits.
        #[arg(short, long, default_value = "niksi.json")]
        config: PathBuf,
        #[arg(short, long, default_value = ".")]
        output_directory: PathBuf,
        #[arg(short, long, default_value = "niksi.lock")]
        lock_file: PathBuf,
        #[arg(short, long, default_value = ".credentials")]
        cred_file: String,
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        verbose: bool,
    },
    Other,
}

fn build(config: Commands) -> Result<(), Box<dyn std::error::Error + 'static>> {
    let Commands::Build {
        push,
        config,
        output_directory,
        lock_file,
        cred_file,
        verbose,
        ..
    } = config
    else {
        unreachable!()
    };

    let niksi = match Niksi::builder()
        .config_file(config.clone())
        .output_directory(output_directory.clone())
        .lock_file(lock_file.clone())
        .build()
    {
        Ok(n) => n,
        Err(BuilderError::NoSuchFile(_)) => {
            eprintln!(
                "No configuration file found at {config:#?}. Generating sample configuration"
            );

            let mut file = std::fs::File::create(config)?;
            file.write_all(include_bytes!("../assets/sample-niksi.json"))?;

            // FIXME: this should still probably be an error
            return Ok(());
        }
        _ => todo!(),
    };

    info!("Building Niksi docker image");
    let result = niksi.build(verbose)?;
    info!("Build result in {:#?}", result);

    if push {
        info!("Pushing image");
        match niksi.push(result, cred_file) {
            Err(_) => Err(BuilderError::PushError),
            _ => Ok(()),
        }?;
        info!("Pushing successful");
    }

    info!("Generating .devcontainer.json");
    let devcontainer = niksi.devcontainer_json()?;
    let mut file = std::fs::File::create(output_directory.as_path().join(".devcontainer.json"))?;
    file.write_all(devcontainer.as_bytes())?;

    Ok(())
}

fn main() -> Result<(), Box<(dyn std::error::Error + 'static)>> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let args = Args::parse();

    match args.command {
        cfg @ Commands::Build { .. } => build(cfg)?,
        _ => todo!(),
    }

    Ok(())
}
