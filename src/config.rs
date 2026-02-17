use std::path::PathBuf;

pub struct Config {
    pub default_category: String,
    pub package_manager: String,
    pub pacman_log_file: PathBuf,
    pub dry_run: bool,
    pub backup_dir: String,
}
