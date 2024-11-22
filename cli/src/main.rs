use clap::{Parser, Subcommand};
use std::path::PathBuf;

use lib::{BuilderError, Niksi};

use std::io::Write;

#[derive(Parser, Debug, Clone)]
#[command(version, about)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    Build {
        /// Location of the config file.
        /// If no such file exists the program creates a sample configuration file in that
        /// location and then exits.
        #[arg(short, long, default_value = "niksi.json")]
        config: PathBuf,
        #[arg(short, long, default_value = ".")]
        output_directory: PathBuf,
    },
    Other,
}

fn build(config: Commands) -> Result<(), Box<dyn std::error::Error + 'static>> {
    let Commands::Build {
        config,
        output_directory,
        ..
    } = config
    else {
        unreachable!()
    };

    let niksi = match Niksi::builder()
        .config_file(config.clone())
        .output_directory(output_directory)
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

    let devcontainer = niksi.devcontainer_json()?;
    // FIXME: use working_directory
    let mut file = std::fs::File::create(".devcontainer.json")?;
    file.write_all(devcontainer.as_bytes())?;

    Ok(())
}

fn main() -> Result<(), Box<(dyn std::error::Error + 'static)>> {
    let args = Args::parse();
    println!("Running command {:#?}", args.command);

    match args.command {
        cfg @ Commands::Build { .. } => build(cfg)?,
        _ => todo!(),
    }

    Ok(())
}
