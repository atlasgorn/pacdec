mod app;
mod cli;
mod commands;
mod kdl_edit;
mod list_pkgs;
mod pacman;
mod parse_kdl;
mod prompts;

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use commands::*;

use crate::list_pkgs::collect_documents;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut app = app::App {
        docs: collect_documents(&cli.declare)?,
        config: app::Config {
            pacman_log_file: cli.pacman_log_file.clone(),
            default_category: "uncat".to_string(),
        },
    };
    match cli.command {
        cli::Commands::Sync(_) => sync_cmd(&mut app, &cli)?,
        cli::Commands::Generate(_) => gen_cmd(&mut app, &cli)?,
        cli::Commands::Add(ref args) => add_cmd(&mut app, &cli, args)?,
        cli::Commands::Remove(ref args) => remove_cmd(&mut app, &cli, args)?,
        cli::Commands::Install(ref args) => install_cmd(&mut app, &cli, args)?,
        cli::Commands::Uninstall(ref args) => uninstall_cmd(&mut app, &cli, args)?,
        cli::Commands::Search(ref args) => search_cmd(&mut app, &cli, args)?,
        _ => eprintln!("Not implemented yet"),
    }

    Ok(())
}
