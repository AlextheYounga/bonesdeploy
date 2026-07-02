use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use shared::config;
use shared::config::runtime_group_for;

use super::{release_directory, staged_release_name, tree};

pub(super) fn run(site: &str, context: &Path, cfg: &config::Bones) -> Result<PathBuf> {
    let release_name = staged_release_name(site)?;
    let release_dir = release_directory(&cfg.project_root, &release_name);
    let release_group = runtime_group_for(&cfg.project_name);
    tree::harden_release_tree(context, &release_dir, &release_group)
        .with_context(|| format!("Failed to promote release {release_name}"))?;

    println!("Promoted release {release_name} into {}", release_dir.display());
    Ok(release_dir)
}
