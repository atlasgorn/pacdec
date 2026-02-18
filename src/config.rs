use std::path::PathBuf;

pub struct Config {
    pub default_category: String,
    pub package_manager: String,
    pub pacman_log_file: PathBuf,
    pub dry_run: bool,
    pub backup_dir: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_category: "uncat".to_string(),
            package_manager: "paru".into(),
            pacman_log_file: "/var/log/pacman.log".into(),
            dry_run: false,
            backup_dir: ".backups".into(),
        }
    }
}
