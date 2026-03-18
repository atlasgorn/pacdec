mod app;
mod cli;
mod commands;
mod config;
mod kdl_edit;
mod list_pkgs;
mod packages;
mod pacman;
mod prompts;

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use colored::Colorize;
use commands::*;

use crate::app::App;

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    let mut app = App::init(&cli)?;

    match &cli.command {
        cli::Commands::Sync(_) => sync_cmd(&app)?,
        cli::Commands::Generate(_) => gen_cmd(&mut app)?,
        cli::Commands::Add(_) => add_cmd(&mut app, cli, false)?,
        cli::Commands::Remove(_) => remove_cmd(&mut app, cli, false)?,
        cli::Commands::Install(_) => add_cmd(&mut app, cli, true)?,
        cli::Commands::Uninstall(_) => remove_cmd(&mut app, cli, true)?,
        cli::Commands::Search(args) => search_cmd(&app, args)?,
        _ => unimplemented!(),
    }

    Ok(())
}
