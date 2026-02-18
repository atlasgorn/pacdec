use std::{fs, path::PathBuf};

use anyhow::Result;

use crate::{
    cli::{self, Cli},
    config::Config,
    list_pkgs::collect_documents,
};
use colored::*;

use kdl::KdlDocument;

pub struct App {
    pub docs: Vec<(PathBuf, KdlDocument)>,
    pub config: Config,
}

impl App {
    pub fn init(cli: &Cli) -> Result<Self> {
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
        Ok(App {
            docs: collect_documents(&declare_file)?,
            config,
        })
    }
}
