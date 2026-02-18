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

    let config_file = cli
        .config
        .clone()
        .or_else(|| std::env::var("PACDEC_CONFIG").ok().map(PathBuf::from))
        .unwrap_or_else(|| {
            shellexpand::tilde("~/.config/pacdec/config.kdl")
                .as_ref()
                .into()
        });

    if !config_file.exists() {
        match cli.command {
            cli::Commands::Generate(_) => {
                if inquire::Confirm::new(&format!(
                    "Configuration file not found at {}. Do you want to create it?",
                    config_file.display()
                ))
                .with_default(true)
                .prompt()?
                {
                    if let Some(parent) = config_file.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::write(&config_file, "")?; // TODO: default config content
                    println!("Created declaration file at {}", config_file.display());
                }
            }
            _ => {
                eprintln!(
                    "{} {}\nPlease create it with {} or choose different file with --cfg or PACDEC_CONFIG environment variable.",
                    "Config file not found at".red(),
                    config_file.display().to_string().italic(),
                    "pacdec gen".blue().bold(),
                );
                std::process::exit(1);
            }
        }
    }

    let config = Config::default(); // TODO: parse from file

    let declare_file = cli
        .declare
        .clone()
        .or_else(|| std::env::var("PACDEC_DECLARE").ok().map(PathBuf::from))
        .unwrap_or_else(|| {
            shellexpand::tilde("~/.config/pacdec/packages.kdl")
                .as_ref()
                .into()
        });

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
