use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use bonesdeploy_core::paths;

pub fn refresh_local_infrastructure(source_dir: &Path, project_root: &Path, template: Option<&str>) -> Result<()> {
    let infra_dir = project_root.join(paths::LOCAL_INFRA_DIR);
    if !infra_dir.exists() {
        return Ok(());
    }

    check_managed_core_conflicts(source_dir, project_root, template)?;
    sync_kit_deployment_functions(source_dir, &infra_dir)?;
    sync_tree(&deployment_source_root(source_dir, template), &infra_dir.join("deployment"), true)?;
    sync_managed_core(source_dir, project_root, template)?;

    Ok(())
}

fn sync_kit_deployment_functions(source_dir: &Path, infra_dir: &Path) -> Result<()> {
    let source = source_dir.join("crates/bonesdeploy/assets/kit/deployment/functions.sh");
    if source.is_file() {
        copy_file(&source, &infra_dir.join("deployment/functions.sh"), true)?;
    }
    Ok(())
}

fn deployment_source_root(source_dir: &Path, template: Option<&str>) -> PathBuf {
    let Some(template) = template else {
        return source_dir.join("crates/bonesdeploy/assets/kit/deployment");
    };

    let framework_deployment =
        source_dir.join("crates/bonesdeploy/assets/frameworks").join(template).join("deployment");
    if framework_deployment.is_dir() {
        framework_deployment
    } else {
        source_dir.join("crates/bonesdeploy/assets/kit/deployment")
    }
}

fn sync_managed_core(source_dir: &Path, project_root: &Path, template: Option<&str>) -> Result<()> {
    let template = template.unwrap_or("custom");
    let source = source_dir.join("crates/bonesinfra/python/src/bonesinfra/frameworks").join(template);
    if !source.is_dir() {
        return Ok(());
    }

    let destination = project_root.join(paths::LOCAL_INFRA_DIR).join("provision/core");
    check_tree_conflicts(&source, &destination)?;
    sync_tree(&source, &destination, false)
}

fn check_managed_core_conflicts(source_dir: &Path, project_root: &Path, template: Option<&str>) -> Result<()> {
    let template = template.unwrap_or("custom");
    let source = source_dir.join("crates/bonesinfra/python/src/bonesinfra/frameworks").join(template);
    if !source.is_dir() {
        return Ok(());
    }

    let destination = project_root.join(paths::LOCAL_INFRA_DIR).join("provision/core");
    check_tree_conflicts(&source, &destination)
}

fn check_tree_conflicts(source_root: &Path, dest_root: &Path) -> Result<()> {
    if !dest_root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(source_root).with_context(|| format!("Failed to read {}", source_root.display()))? {
        let entry = entry.with_context(|| format!("Failed to read entry in {}", source_root.display()))?;
        let source_path = entry.path();
        let dest_path = dest_root.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            check_tree_conflicts(&source_path, &dest_path)?;
        } else if dest_path.is_file() && fs::read(&source_path)? != fs::read(&dest_path)? {
            bail!("Managed infrastructure conflict at {}; refusing to overwrite it", dest_path.display());
        }
    }
    Ok(())
}

fn sync_tree(source_root: &Path, dest_root: &Path, executable: bool) -> Result<()> {
    if !source_root.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(source_root).with_context(|| format!("Failed to read {}", source_root.display()))? {
        let entry = entry.with_context(|| format!("Failed to read entry in {}", source_root.display()))?;
        let file_type =
            entry.file_type().with_context(|| format!("Failed to read file type for {}", entry.path().display()))?;
        let source_path = entry.path();
        let dest_path = dest_root.join(entry.file_name());

        if file_type.is_dir() {
            fs::create_dir_all(&dest_path).with_context(|| format!("Failed to create {}", dest_path.display()))?;
            sync_tree(&source_path, &dest_path, executable)?;
            continue;
        }

        copy_file(&source_path, &dest_path, executable)?;
    }

    Ok(())
}

fn copy_file(source: &Path, dest: &Path, executable: bool) -> Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    fs::copy(source, dest).with_context(|| format!("Failed to copy {} to {}", source.display(), dest.display()))?;

    if executable {
        fs::set_permissions(dest, fs::Permissions::from_mode(0o755))
            .with_context(|| format!("Failed to set permissions on {}", dest.display()))?;
    }

    Ok(())
}
