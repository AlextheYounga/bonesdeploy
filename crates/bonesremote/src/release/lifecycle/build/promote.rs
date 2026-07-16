use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use shared::config;
use shared::config::{load_runtime, runtime_group_for, runtime_user_for};
use shared::paths;

use super::{release_directory, staged_release_name, tree};

pub(super) fn run(site: &str, context: &Path, cfg: &config::Bones) -> Result<PathBuf> {
    let release_name = staged_release_name(site)?;
    let release_dir = release_directory(&cfg.project_root, &release_name);
    let runtime = load_runtime(&paths::bonesremote_site_root(site))
        .with_context(|| format!("Failed to load runtime configuration for {site}"))?;
    let runtime_user =
        if runtime.runtime_user.is_empty() { runtime_user_for(&cfg.project_name) } else { runtime.runtime_user };
    let runtime_group =
        if runtime.runtime_group.is_empty() { runtime_group_for(&cfg.project_name) } else { runtime.runtime_group };
    tree::prepare_release_tree(context, &release_dir, &runtime_user, &runtime_group)
        .with_context(|| format!("Failed to promote release {release_name}"))?;

    println!("Copied release {release_name} into {}", release_dir.display());
    Ok(release_dir)
}

pub(super) fn finalize(site: &str, cfg: &config::Bones) -> Result<()> {
    let release_name = staged_release_name(site)?;
    let release_dir = release_directory(&cfg.project_root, &release_name);
    let runtime = load_runtime(&paths::bonesremote_site_root(site))
        .with_context(|| format!("Failed to load runtime configuration for {site}"))?;
    let runtime_group =
        if runtime.runtime_group.is_empty() { runtime_group_for(&cfg.project_name) } else { runtime.runtime_group };

    tree::seal_release_tree(&release_dir, &runtime_group)
        .with_context(|| format!("Failed to seal release {release_name}"))?;
    println!("Sealed release {release_name}.");
    Ok(())
}
