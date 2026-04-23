use std::path::Path;

use anyhow::Result;

use crate::config;
use crate::permissions;

pub fn run(config_path: &str) -> Result<()> {
    let cfg = config::load(Path::new(config_path))?;
    permissions::harden(&cfg)
}
