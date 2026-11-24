//! Native RustInfra provisioning and project template materialization.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use bonesdeploy_core::config::Runtime;
use bonesdeploy_core::paths;
use rust_embed::Embed;

mod context;
mod execution;
mod framework_paths;
pub mod frameworks;
pub mod manifest;
pub mod patches;
pub mod provisioning;
mod shell_quote;
mod templates;

pub use context::{ProvisioningContext, ServerContext};
pub use execution::{HostProvisioningReport, ProvisioningReport, ProvisioningRuntime};
pub use templates::TemplateResolver;

#[derive(Embed)]
#[folder = "templates/"]
struct TemplateAsset;

const PREVIOUS_FRAMEWORK_PATH: &str = "infra/.framework";

/// Replaces the managed project-local template snapshot and prunes templates
/// for unselected frameworks. Secrets and deployment inputs are untouched.
///
/// # Errors
/// Fails for existing custom provisioning, an unknown framework, or filesystem
/// errors while atomically replacing the managed template directory.
pub fn materialize_project_artifacts(project_root: &Path, runtime: &Runtime) -> Result<PathBuf> {
    reject_custom_provisioning(project_root)?;
    let infra = project_root.join(paths::LOCAL_INFRA_DIR);
    fs::create_dir_all(&infra).with_context(|| format!("Failed to create {}", infra.display()))?;
    remove_project_wheels(&infra)?;
    let templates = project_root.join(paths::LOCAL_INFRA_TEMPLATES_DIR);
    replace_templates(&templates)?;
    framework_paths::prune(&infra, runtime)?;
    remove_path(&project_root.join(PREVIOUS_FRAMEWORK_PATH))?;
    Ok(templates)
}

/// Stops migration before managed project files are changed when the removed
/// Python extension point is present.
///
/// # Errors
/// Returns a migration explanation when `infra/custom/` exists.
pub fn reject_custom_provisioning(project_root: &Path) -> Result<()> {
    let custom = project_root.join(paths::LOCAL_INFRA_DIR).join("custom");
    if custom.exists() || custom.is_symlink() {
        bail!(
            "Custom provisioning at {} is temporarily unsupported by Rust BonesInfra. Archive or remove this directory deliberately, then run the command again; BonesDeploy will not delete or ignore it.",
            custom.display()
        );
    }
    Ok(())
}

/// Returns embedded template paths relative to `infra/templates/`.
pub fn embedded_template_paths() -> impl Iterator<Item = String> {
    TemplateAsset::iter().map(|path| path.into_owned())
}

fn replace_templates(destination: &Path) -> Result<()> {
    let parent = destination.parent().context("template directory has no parent")?;
    let staging = tempfile::Builder::new().prefix(".templates.").tempdir_in(parent)?;
    for relative_path in embedded_template_paths() {
        let Some(asset) = TemplateAsset::get(&relative_path) else {
            continue;
        };
        let target = staging.path().join(relative_path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&target, asset.data.as_ref())?;
    }
    replace_directory(destination, staging)
}

fn replace_directory(destination: &Path, staging: tempfile::TempDir) -> Result<()> {
    let previous = destination.with_extension("previous");
    remove_path(&previous)?;
    if destination.exists() || destination.is_symlink() {
        fs::rename(destination, &previous)?;
    }
    if let Err(error) = fs::rename(staging.keep(), destination) {
        if previous.exists() {
            let _ = fs::rename(&previous, destination);
        }
        return Err(error).with_context(|| format!("Failed to replace {}", destination.display()));
    }
    remove_path(&previous)
}

fn remove_project_wheels(infra: &Path) -> Result<()> {
    for entry in fs::read_dir(infra)? {
        let path = entry?.path();
        if path
            .file_name()
            .is_some_and(|name| name == "bonesinfra.whl" || name.to_string_lossy().starts_with("bonesinfra-"))
            && path.extension().is_some_and(|extension| extension == "whl")
        {
            remove_path(&path)?;
        }
    }
    Ok(())
}

fn remove_path(path: &Path) -> Result<()> {
    if let Ok(metadata) = fs::symlink_metadata(path) {
        if metadata.file_type().is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}
