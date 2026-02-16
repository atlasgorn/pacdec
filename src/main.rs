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

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        cli::Commands::Sync(_) => sync_cmd(&cli)?,
        cli::Commands::Generate(_) => gen_cmd(&cli)?,
        cli::Commands::Add(ref args) => add_cmd(&cli, args)?,
        cli::Commands::Remove(ref args) => remove_cmd(&cli, args)?,
        cli::Commands::Search(ref args) => search_cmd(&cli, args)?,
        _ => eprintln!("Not implemented yet"),
    }

    Ok(())
}
