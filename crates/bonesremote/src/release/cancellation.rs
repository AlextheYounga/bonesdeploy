use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use bonesdeploy_core::paths;

use crate::release::lifecycle::context;
use crate::release::{SiteMutation, failed};
use crate::runtime::docker::command::release_image_tags;

/// Finishes a validated cancellation while holding the site mutation lock.
/// The caller must first stop any live deployment process for this release.
pub fn finish_locked(mutation: &SiteMutation, release: &str) -> Result<()> {
    let site = mutation.site();
    let current = mutation.active()?;
    if current.as_ref().is_some_and(|deployment| deployment.release() != release) {
        bail!("Active deployment changed while cancelling {release}; no cleanup was performed.");
    }

    if let Some(context) = current.as_ref().and_then(|deployment| deployment.context()) {
        let context = validate_build_context(site, &mutation.config().project_root, Path::new(context))?;
        let tags = release_image_tags(&context)?;
        failed::remove_unreferenced_manifest_images(mutation, &tags, Some(release))?;
        context::cleanup(site, &context)?;
    } else {
        cleanup_stale_contexts(site, &mutation.config().project_root, mutation)?;
    }

    let staged = mutation.staged_release()?;
    if staged.as_deref() == Some(release) {
        failed::run_locked(mutation)?;
    }
    mutation.clear_active()?;
    Ok(())
}

fn validate_build_context(site: &str, project_root: &str, context: &Path) -> Result<PathBuf> {
    let tmp_root = fs::canonicalize(Path::new(project_root).join(paths::TMP_BUILDS_DIR))
        .with_context(|| format!("Failed to resolve build context root for release {site}"))?;
    let metadata = fs::symlink_metadata(context)
        .with_context(|| format!("Failed to inspect recorded build context {}", context.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("Refusing to remove invalid build context recorded for release {site}: {}", context.display());
    }
    let canonical = fs::canonicalize(context)
        .with_context(|| format!("Failed to resolve recorded build context {}", context.display()))?;
    if canonical != context
        || !canonical.starts_with(&tmp_root)
        || !canonical.file_name().is_some_and(|name| name.to_string_lossy().starts_with(&format!("build-{site}-")))
    {
        bail!("Refusing to remove invalid build context recorded for release {site}: {}", context.display());
    }
    Ok(canonical)
}

fn cleanup_stale_contexts(site: &str, project_root: &str, mutation: &SiteMutation) -> Result<()> {
    let tmp_root = Path::new(project_root).join(paths::TMP_BUILDS_DIR);
    if !tmp_root.is_dir() {
        return Ok(());
    }
    let tmp_root = fs::canonicalize(&tmp_root)
        .with_context(|| format!("Failed to resolve build context root for release {site}"))?;
    for entry in fs::read_dir(&tmp_root).with_context(|| format!("Failed to read {}", tmp_root.display()))? {
        let path = entry?.path();
        if !path.file_name().is_some_and(|name| name.to_string_lossy().starts_with(&format!("build-{site}-"))) {
            continue;
        }
        let metadata = fs::symlink_metadata(&path)?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            continue;
        }
        let canonical = fs::canonicalize(&path)?;
        if canonical != path || !canonical.starts_with(&tmp_root) {
            continue;
        }
        let tags = release_image_tags(&canonical)?;
        failed::remove_unreferenced_manifest_images(mutation, &tags, None)?;
        context::cleanup(site, &canonical)?;
    }
    Ok(())
}
