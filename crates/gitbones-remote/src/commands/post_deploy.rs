use std::path::Path;

use anyhow::Result;

use crate::config;
use crate::permissions;

pub fn run(config_path: &str) -> Result<()> {
    let cfg = config::load(Path::new(config_path))?;

    let current_link = Path::new(&cfg.data.worktree).join("current");
    if current_link.is_symlink() {
        permissions::harden_release(&cfg)
    } else {
        permissions::harden(&cfg)
    }
}
