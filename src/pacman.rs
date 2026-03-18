use anyhow::Result;
use std::process::{Command, ExitStatus};

use crate::config::Config;

pub fn run_pacman(cfg: &Config, args: &[&str]) -> Result<String> {
    let output = Command::new(&cfg.package_manager).args(args).output()?;
    if output.status.code() != Some(0) {
        return Err(anyhow::anyhow!(
            "pacman command failed with code {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let packages = String::from_utf8(output.stdout)?;
    Ok(packages)
}

pub fn sudo_pacman(cfg: &Config, args: &[&str], pkgs: &[String]) -> Result<ExitStatus> {
    let mut cmd = if cfg.package_manager == "pacman" {
        let mut cmd = std::process::Command::new("sudo");
        cmd.arg("pacman");
        cmd
    } else {
        std::process::Command::new(&cfg.package_manager)
    };

    cmd.args(args).args(pkgs).status().map_err(Into::into)
}

pub fn check_pkg_exists(cfg: &Config, pkg: &str) -> bool {
    run_pacman(cfg, &["-Si", pkg]).is_ok()
}
