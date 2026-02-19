use std::path::PathBuf;

pub struct Config {
    pub declaration_file: PathBuf,
    pub pacman_log_file: PathBuf,
    pub default_category: String,
    pub package_manager: String,
    pub dry_run: bool,
    pub backup: BackupConfig,
    pub packages: PackagesConfig,
}

#[derive(Default)]
pub struct PackagesConfig {
    pub whitelist: Vec<String>,
    pub blacklist: Vec<String>,
}

pub struct BackupConfig {
    pub dir: String,
    pub mode: BackupMode,
}

pub enum BackupMode {
    Off,
    Basic,
    Git,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_category: "uncat".to_string(),
            package_manager: "paru".into(),
            pacman_log_file: "/var/log/pacman.log".into(),
            dry_run: false,
            backup: BackupConfig {
                dir: ".backups".into(),
                mode: BackupMode::Basic,
            },
            declaration_file: shellexpand::tilde("~/.config/pacdec/packages.kdl")
                .as_ref()
                .into(),
            packages: PackagesConfig::default(),
        }
    }
}
