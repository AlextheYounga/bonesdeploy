use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use shared::config;

mod ownership;
mod promote;
mod scripts;
mod tree;

use crate::privileges;
use crate::release::state as release_state;
use shared::paths;

pub fn run(site: &str, context: &Path) -> Result<()> {
    privileges::ensure_root("bonesremote release build")?;
    let cfg = load_site_config(site)?;
    scripts::run(site, context, &cfg)
}

pub fn promote(site: &str, context: &Path) -> Result<PathBuf> {
    privileges::ensure_root("bonesremote release promote")?;
    let cfg = load_site_config(site)?;
    promote::run(site, context, &cfg)
}

pub(super) fn load_site_config(site: &str) -> Result<config::Bones> {
    let bones_path = paths::bonesremote_bones_toml_path(site);
    let cfg = config::load(&bones_path)
        .with_context(|| format!("Failed to load remote site state from {}", bones_path.display()))?;

    if cfg.project_name != site {
        bail!("Remote site state belongs to '{}', expected '{}'", cfg.project_name, site);
    }

    Ok(cfg)
}

pub(super) fn staged_release_name(site: &str) -> Result<String> {
    release_state::read_staged_release(site)
}

pub(super) fn release_directory(project_root: &str, release_name: &str) -> PathBuf {
    release_state::release_dir(project_root, release_name)
}
