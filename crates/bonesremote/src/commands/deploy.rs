use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use shared::config;
use shared::paths;

use crate::commands::{drop_failed_release, release_prune, service};
use crate::privileges;
use crate::release::lifecycle;
use crate::release::state as release_state;

pub fn run_full(site: &str, revision: Option<&str>) -> Result<()> {
    privileges::ensure_root("bonesremote deploy")?;
    let bones_path = paths::bonesremote_bones_toml_path(site);
    let cfg = config::load(&bones_path)
        .with_context(|| format!("Failed to load remote site state from {}", bones_path.display()))?;

    if cfg.project_name != site {
        bail!("Remote site state belongs to '{}', expected '{}'", cfg.project_name, site);
    }

    let target_revision = revision.map_or_else(|| cfg.branch.clone(), ToOwned::to_owned);

    lifecycle::stage::run(site)?;

    let context_dir = lifecycle::checkout::ensure_build_context(site)?;

    if let Err(error) = lifecycle::checkout::run(site, &target_revision, &context_dir) {
        cleanup(site, Some(&context_dir));
        drop_failed_release::run(site).ok();
        return Err(error);
    }

    if let Err(error) = lifecycle::build::run(site, &context_dir) {
        cleanup(site, Some(&context_dir));
        drop_failed_release::run(site).ok();
        return Err(error);
    }

    if let Err(error) = lifecycle::build::promote(site, &context_dir) {
        cleanup(site, Some(&context_dir));
        drop_failed_release::run(site).ok();
        return Err(error);
    }

    if let Err(error) = lifecycle::wire_shared::run(site) {
        cleanup(site, Some(&context_dir));
        drop_failed_release::run(site).ok();
        return Err(error);
    }

    if let Err(error) = lifecycle::prepare::run(site) {
        cleanup(site, Some(&context_dir));
        drop_failed_release::run(site).ok();
        return Err(error);
    }

    if let Err(error) = lifecycle::activate::run(site) {
        cleanup(site, Some(&context_dir));
        drop_failed_release::run(site).ok();
        return Err(error);
    }

    if let Err(error) = service::run(site) {
        cleanup(site, Some(&context_dir));
        drop_failed_release::run(site).ok();
        return Err(error);
    }

    if let Err(error) = release_prune::run(site) {
        cleanup(site, Some(&context_dir));
        return Err(error);
    }

    cleanup(site, Some(&context_dir));
    Ok(())
}

fn cleanup(site: &str, context: Option<&Path>) {
    if let Some(context) = context {
        lifecycle::checkout::cleanup_build_context(site, context).ok();
    }
}

pub fn rollback(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote release rollback")?;
    let bones_path = paths::bonesremote_bones_toml_path(site);
    let cfg = config::load(&bones_path).context("Failed to load remote site state")?;

    if cfg.project_name != site {
        bail!("Remote site state belongs to '{}', expected '{}'", cfg.project_name, site);
    }

    let releases = release_state::list_releases_sorted(&cfg.project_root)?;
    if releases.len() < 2 {
        bail!("Need at least two releases to perform rollback");
    }

    let current_name = release_state::current_release_name(&cfg.project_root)?;
    let current_idx = releases
        .iter()
        .position(|name| name == &current_name)
        .with_context(|| format!("Current release '{current_name}' was not found in releases/"))?;

    if current_idx == 0 {
        bail!("Current release is already the oldest release; cannot roll back");
    }

    let previous_name = releases[current_idx - 1].clone();
    let previous_dir = release_state::release_dir(&cfg.project_root, &previous_name);
    let current_link = PathBuf::from(&cfg.project_root).join(paths::CURRENT_LINK);
    release_state::point_symlink_atomically(&current_link, &previous_dir)?;
    service::run(site)?;

    println!("Rollback complete: {current_name} -> {previous_name}");
    Ok(())
}
