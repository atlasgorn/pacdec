use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

/// Declarative Package Manager
#[derive(Parser, Debug)]
#[command(name = "pacdec")]
#[command(version, about = "Declarative Package Manager", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Configuration file path
    #[arg(
        long = "dec_config",
        alias = "cfg",
        long = "config",
        global = true,
        default_value = "~/.config/pacdec/packages.kdl"
    )]
    pub config: PathBuf,

    /// Path to pacman log file
    #[arg(
        long = "log_file",
        global = true,
        default_value = "/var/log/pacman.log"
    )]
    pub pacman_log_file: PathBuf,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Synchronize packages
    Sync(SyncArgs),

    /// Generate configuration
    #[command(alias = "gen")]
    Generate(GenerateArgs),

    /// Add package(s) to configuration
    Add(AddArgs),

    /// Remove package from configuration
    #[command(alias = "rm")]
    Remove(RemoveArgs),

    /// Install package
    #[command(alias = "ins")]
    Install(InstallArgs),

    /// Uninstall package
    #[command(alias = "unins")]
    Uninstall(UninstallArgs),

    /// Search for packages
    Search(SearchArgs),

    // Revert last chnanges
    #[command(alias = "undo")]
    Revert(RevertArgs),
}

#[derive(Args, Debug)]
pub struct SyncArgs {
    /// Dry run - don't actually sync
    #[arg(short = 'n', long)]
    pub dry_run: bool,

    /// Force sync even if there are warnings
    #[arg(short, long)]
    pub force: bool,
}

#[derive(Args, Debug)]
pub struct GenerateArgs {
    /// Dry run - don't actually generate
    #[arg(long)]
    pub dry_run: bool,

    /// Force generation even if there are warnings
    #[arg(long)]
    pub force: bool,
}

#[derive(Args, Debug)]
pub struct AddArgs {
    /// Package(s) to add
    pub packages: Option<Vec<String>>,

    /// Category for the package
    #[arg(short = 'c', long = "cat")]
    pub category: Option<String>,
}

#[derive(Args, Debug)]
pub struct RemoveArgs {
    /// Package(s) to remove
    pub packages: Option<Vec<String>>,

    /// Comment out packages instead of deleting
    #[arg(long)]
    pub comment: bool,
}

#[derive(Args, Debug)]
pub struct InstallArgs {
    /// Package(s) to install
    pub packages: Option<Vec<String>>,

    /// Category for the package
    #[arg(short = 'c', long = "cat")]
    pub category: Option<String>,
}

#[derive(Args, Debug)]
pub struct UninstallArgs {
    /// Package(s) to uninstall
    pub packages: Option<Vec<String>>,
}

#[derive(Args, Debug)]
pub struct SearchArgs {
    /// Search for explicitly installed packages
    #[arg(short, long)]
    explicit: bool,

    /// Search for all packages in repositories
    #[arg(short, long)]
    all: bool,
}

#[derive(Args, Debug)]
pub struct RevertArgs {}
