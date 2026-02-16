use std::path::PathBuf;

use kdl::KdlDocument;

pub struct App {
    pub docs: Vec<(PathBuf, KdlDocument)>,
    pub config: Config,
}

pub struct Config {
    pub default_category: String,
    pub package_manager: String,
    pub pacman_log_file: PathBuf,
}
