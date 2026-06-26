use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use shared::paths;
use shared::registry;

use crate::commands::{
    activate_release, drop_failed_release, post_deploy, release_build, release_checkout, service, stage_release,
    wire_shared,
};
use crate::privileges;
use crate::release_state;

pub fn run_full(site: &str, revision: Option<&str>) -> Result<()> {
    privileges::ensure_root("bonesremote deploy")?;
    let registry_path = paths::bonesremote_registry_path(site);
    let cfg = registry::load(&registry_path)
        .with_context(|| format!("Failed to load remote site state from {}", registry_path.display()))?;

    let target_revision = revision.map_or_else(|| cfg.branch.clone(), ToOwned::to_owned);

    stage_release::run(site)?;

    let context_dir = release_checkout::ensure_build_context(site)?;

    if let Err(error) = release_checkout::run(site, &target_revision, &context_dir) {
        cleanup(site, Some(&context_dir));
        drop_failed_release::run(site).ok();
        return Err(error);
    }

    if let Err(error) = release_build::run(site, &context_dir) {
        cleanup(site, Some(&context_dir));
        drop_failed_release::run(site).ok();
        return Err(error);
    }

    if let Err(error) = release_build::promote(site, &context_dir) {
        cleanup(site, Some(&context_dir));
        drop_failed_release::run(site).ok();
        return Err(error);
    }

    if let Err(error) = wire_shared::run(site) {
        cleanup(site, Some(&context_dir));
        drop_failed_release::run(site).ok();
        return Err(error);
    }

    if let Err(error) = activate_release::run(site) {
        cleanup(site, Some(&context_dir));
        drop_failed_release::run(site).ok();
        return Err(error);
    }

    if let Err(error) = service::run(site) {
        cleanup(site, Some(&context_dir));
        drop_failed_release::run(site).ok();
        return Err(error);
    }

    if let Err(error) = post_deploy::run(site) {
        cleanup(site, Some(&context_dir));
        return Err(error);
    }

    cleanup(site, Some(&context_dir));
    Ok(())
}

fn cleanup(site: &str, context: Option<&Path>) {
    if let Some(context) = context {
        release_checkout::cleanup_build_context(site, context).ok();
    }
}

pub fn rollback(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote release rollback")?;
    let cfg = registry::load(&paths::bonesremote_registry_path(site)).context(registry_load_error())?;

    let releases = release_state::list_releases_sorted(&cfg)?;
    if releases.len() < 2 {
        bail!("Need at least two releases to perform rollback");
    }

    let current_name = release_state::current_release_name(&cfg)?;
    let current_idx = releases
        .iter()
        .position(|name| name == &current_name)
        .with_context(|| format!("Current release '{current_name}' was not found in releases/"))?;

    if current_idx == 0 {
        bail!("Current release is already the oldest release; cannot roll back");
    }

    let previous_name = releases[current_idx - 1].clone();
    let previous_dir = release_state::release_dir(&cfg, &previous_name);
    let current_link = PathBuf::from(&cfg.current_path);
    release_state::point_symlink_atomically(&current_link, &previous_dir)?;
    service::run(site)?;

    println!("Rollback complete: {current_name} -> {previous_name}");
    Ok(())
}

pub fn registry_load_error() -> String {
    String::from("Failed to load remote site state from registry")
}
