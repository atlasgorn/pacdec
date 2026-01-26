use anyhow::Result;
use std::process::{Command, ExitStatus};

pub fn run_pacman(args: &[&str]) -> Result<String> {
    let output = Command::new("pacman").args(args).output()?; // TODO: aur helpers
    let packages = String::from_utf8(output.stdout)?;
    Ok(packages)
}

pub fn sudo_pacman(args: &[&str], pkgs: &[String]) -> Result<ExitStatus> {
    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("pacman").args(args).args(pkgs);

    Ok(cmd.status()?)
}
