use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

use crate::packages::{Category, Package};

/// Declarative Package Manager
#[derive(Parser, Debug)]
#[command(name = "pacdec")]
#[command(version, about = "Declarative Package Manager For Arch", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Configuration file path
    #[arg(alias = "cfg", long = "config", global = true, env = "PACDEC_CONFIG")]
    pub config: Option<PathBuf>,

    /// Declaration file path
    #[arg(alias = "dec", long = "declare", global = true, env = "PACDEC_DECLARE")]
    pub declare: Option<PathBuf>,

    /// Path to pacman log file
    #[arg(long = "log-file", global = true)]
    pub pacman_log_file: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Synchronize system state with declaration file
    Sync(SyncArgs),

    /// Generate declaration file or synchronize declaration file with system state (alias: gen)
    #[command(alias = "gen")]
    Generate(GenerateArgs),

    /// Add package(s) to configuration
    Add(AddArgs),

    /// Remove package(s) from configuration (alias: rm)
    #[command(alias = "rm")]
    Remove(RemoveArgs),

    /// Install package(s) and add package(s) to configuration (alias: ins)
    #[command(alias = "ins")]
    Install(InstallArgs),

    /// Uninstall package(s) and remove package(s) from configuration (alias: unins)
    #[command(alias = "unins")]
    Uninstall(UninstallArgs),

    /// Interactive search for packages. For installed packages if no flags specified
    Search(SearchArgs),

    /// Revert last changes (alias: undo)
    #[command(alias = "undo")]
    Revert(RevertArgs),
}

#[derive(Args, Debug)]
pub struct SyncArgs {
    /// Dry run, only show changes
    #[arg(short = 'n', long)]
    pub dry_run: bool,

    /// Force sync, ignore warnings
    #[arg(short, long)]
    pub force: bool,
}

#[derive(Args, Debug)]
pub struct GenerateArgs {
    /// Dry run, only show changes
    #[arg(short = 'n', long)]
    pub dry_run: bool,

    /// Force sync, ignore warnings
    #[arg(short, long)]
    pub force: bool,
}

#[derive(Args, Debug)]
pub struct AddArgs {
    /// Package(s) to add (interactive picker if omitted)
    pub packages: Option<Vec<Package>>,

    /// Category for the package (interactive picker if omitted)
    #[arg(short = 'c', long = "cat")]
    pub category: Option<Category>,

    /// Tags for the package
    #[arg(short = 't', long = "tag")]
    pub tags: Option<Vec<String>>,
}

#[derive(Args, Debug)]
pub struct InstallArgs {
    /// Package(s) to install (interactive picker if omitted)
    pub packages: Option<Vec<Package>>,

    /// Category for the package (interactive picker if omitted)
    #[arg(short = 'c', long = "cat")]
    pub category: Option<Category>,

    /// Tags for the package
    #[arg(short = 't', long = "tag")]
    pub tags: Option<Vec<String>>,
}

#[derive(Args, Debug)]
pub struct RemoveArgs {
    /// Package(s) to remove (interactive picker if omitted)
    pub packages: Option<Vec<Package>>,

    /// Comment out package(s) instead of deleting
    #[arg(long)]
    pub comment: bool,
}

#[derive(Args, Debug)]
pub struct UninstallArgs {
    /// Package(s) to uninstall (interactive picker if omitted)
    pub packages: Option<Vec<Package>>,

    /// Comment out package(s) instead of deleting
    #[arg(long)]
    pub comment: bool,
}

#[derive(Args, Debug)]
pub struct SearchArgs {
    /// Search for explicitly installed packages
    #[arg(short, long)]
    pub explicit: bool,

    /// Search for all packages in repositories
    #[arg(short, long)]
    pub all: bool,

    /// List explicitly installed packages ordered by time using pacman log
    #[arg(short, long)]
    pub chronological: bool,
}

#[derive(Args, Debug)]
pub struct RevertArgs {}
