mod app;
mod cli;
mod commands;
mod config;
mod kdl_edit;
mod list_pkgs;
mod pacman;
mod prompts;

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use commands::*;

use crate::app::App;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut app = App::init(&cli)?;

    match cli.command {
        cli::Commands::Sync(_) => sync_cmd(&app)?,
        cli::Commands::Generate(_) => gen_cmd(&mut app)?,
        cli::Commands::Add(_) => add_cmd(&mut app, &cli)?,
        cli::Commands::Remove(ref args) => remove_cmd(&mut app, args)?,
        cli::Commands::Install(_) => install_cmd(&mut app, &cli)?,
        cli::Commands::Uninstall(ref args) => uninstall_cmd(&mut app, args)?,
        cli::Commands::Search(ref args) => search_cmd(&app, args)?,
        _ => eprintln!("Not implemented yet"),
    }

    Ok(())
}
