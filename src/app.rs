use std::path::PathBuf;

use kdl::KdlDocument;

use crate::config::Config;

pub struct App {
    pub docs: Vec<(PathBuf, KdlDocument)>,
    pub config: Config,
}
