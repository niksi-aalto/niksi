use std::process::{Command, Stdio};

use crate::BuilderError;

pub fn push(
    location: String,
    name: String,
    version: String,
    registry: String,
    creds: String,
) -> Result<(), BuilderError> {
    Command::new("skopeo")
        .args([
            "--insecure-policy",
            "copy",
            &format!("docker-archive:{location}"),
            &format!("docker://{registry}/{name}:{version}"),
            &format!("--dest-creds={creds}"),
        ])
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .status()
        .map_err(|_| BuilderError::PushError)?;

    Ok(())
}
