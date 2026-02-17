mod app;
mod cli;
mod commands;
mod config;
mod kdl_edit;
mod list_pkgs;
mod pacman;
mod prompts;

use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use commands::*;

use crate::{app::App, config::Config, list_pkgs::collect_documents};
use colored::*;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let config = Config {
        pacman_log_file: cli
            .pacman_log_file
            .clone()
            .unwrap_or("/var/log/pacman.log".into()),
        default_category: "uncat".to_string(),
        package_manager: "paru".into(),
        dry_run: false,
        backup_dir: ".backups".into(),
    };

    let declare_file = {
        let declare_file = &cli
            .declare
            .clone()
            .unwrap_or("~/.config/pacdec/packages.kdl".into());
        let path_str = &declare_file.to_string_lossy();
        let expanded_path = shellexpand::tilde(path_str);
        PathBuf::from(expanded_path.as_ref())
    };

    if !declare_file.exists() {
        match cli.command {
            cli::Commands::Generate(_) => {
                let should_create = inquire::Confirm::new(&format!(
                    "Declaration file not found at {}. Do you want to create it?",
                    declare_file.display()
                ))
                .with_default(true)
                .prompt()?;

                if should_create {
                    if let Some(parent) = declare_file.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::write(&declare_file, format!("cat:{}", config.default_category))?;
                    println!("Created declaration file at {}", declare_file.display());
                }
            }
            _ => {
                eprintln!(
                    "{} {}\nPlease choose different file or run {} to create it.",
                    "Declaration file not found at".red(),
                    declare_file.display().to_string().italic(),
                    "pacdec gen".blue().bold()
                );
                std::process::exit(1);
            }
        }
    }

    let mut app = App {
        docs: collect_documents(&declare_file)?,
        config,
    };

    match cli.command {
        cli::Commands::Sync(_) => sync_cmd(&mut app)?,
        cli::Commands::Generate(_) => gen_cmd(&mut app)?,
        cli::Commands::Add(ref args) => add_cmd(&mut app, &cli, args)?,
        cli::Commands::Remove(ref args) => remove_cmd(&mut app, args)?,
        cli::Commands::Install(ref args) => install_cmd(&mut app, &cli, args)?,
        cli::Commands::Uninstall(ref args) => uninstall_cmd(&mut app, args)?,
        cli::Commands::Search(ref args) => search_cmd(&mut app, args)?,
        _ => eprintln!("Not implemented yet"),
    }

    Ok(())
}
