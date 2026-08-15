use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use bonesdeploy_core::config::project_env;
use bonesdeploy_core::paths;

pub(super) fn refresh_local_infrastructure(source_dir: &Path, project_root: &Path) -> Result<()> {
    let infra_dir = project_root.join(paths::LOCAL_INFRA_DIR);
    if !infra_dir.exists() {
        return Ok(());
    }

    sync_kit_deployment_functions(source_dir, &infra_dir)?;
    sync_tree(&deployment_source_root(source_dir, project_root)?, &infra_dir.join("deployment"), true)?;
    sync_managed_core(source_dir, project_root)?;

    Ok(())
}

fn sync_kit_deployment_functions(source_dir: &Path, infra_dir: &Path) -> Result<()> {
    let source = source_dir.join("crates/bonesdeploy/assets/kit/deployment/functions.sh");
    if source.is_file() {
        copy_file(&source, &infra_dir.join("deployment/functions.sh"), true)?;
    }
    Ok(())
}

fn deployment_source_root(source_dir: &Path, project_root: &Path) -> Result<PathBuf> {
    let Some(template) = selected_framework_template(&project_root.join(paths::DOT_ENV))? else {
        return Ok(source_dir.join("crates/bonesdeploy/assets/kit/deployment"));
    };

    let framework_deployment =
        source_dir.join("crates/bonesdeploy/assets/frameworks").join(template).join("deployment");
    Ok(if framework_deployment.is_dir() {
        framework_deployment
    } else {
        source_dir.join("crates/bonesdeploy/assets/kit/deployment")
    })
}

fn sync_managed_core(source_dir: &Path, project_root: &Path) -> Result<()> {
    let template =
        selected_framework_template(&project_root.join(paths::DOT_ENV))?.unwrap_or_else(|| String::from("custom"));
    let source = source_dir.join("crates/bonesinfra/python/src/bonesinfra/frameworks").join(&template);
    if !source.is_dir() {
        return Ok(());
    }

    let destination = project_root.join(paths::LOCAL_INFRA_DIR).join("provision/core");
    check_tree_conflicts(&source, &destination)?;
    sync_tree(&source, &destination, false)
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

fn selected_framework_template(framework_toml: &Path) -> Result<Option<String>> {
    let content =
        fs::read_to_string(framework_toml).with_context(|| format!("Failed to read {}", framework_toml.display()))?;
    Ok(content.lines().find_map(|line| {
        let (key, value) = line.split_once('=')?;
        (key.trim() == project_env::TEMPLATE).then(|| value.trim().trim_matches(['"', '\'']).to_string())
    }))
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;

    use super::refresh_local_infrastructure;
    use anyhow::Result;
    use tempfile::TempDir;

    #[test]
    fn refresh_local_infrastructure_updates_managed_files_without_touching_custom() -> Result<()> {
        let temp = TempDir::new()?;
        let source_dir = temp.path().join("source");
        let project_root = temp.path().join("project");
        let infra_dir = project_root.join("infra");
        fs::create_dir_all(&infra_dir)?;

        write(&source_dir.join("crates/bonesdeploy/assets/kit/deployment/build/01_build.sh"), "generic deploy")?;
        write(&source_dir.join("crates/bonesdeploy/assets/kit/deployment/functions.sh"), "shared functions")?;
        write(
            &source_dir.join("crates/bonesdeploy/assets/frameworks/laravel/deployment/build/01_build.sh"),
            "laravel deploy",
        )?;

        write(&project_root.join(".env"), "TEMPLATE=laravel\n")?;
        write(&infra_dir.join("provision/custom/runtime.py"), "def deploy(ctx):\n    custom(ctx)\n")?;

        refresh_local_infrastructure(&source_dir, &project_root)?;

        assert_eq!(fs::read_to_string(project_root.join(".env"))?, "TEMPLATE=laravel\n");
        assert_eq!(fs::read_to_string(infra_dir.join("deployment/build/01_build.sh"))?, "laravel deploy");
        assert_eq!(fs::read_to_string(infra_dir.join("deployment/functions.sh"))?, "shared functions");
        assert_eq!(
            fs::read_to_string(infra_dir.join("provision/custom/runtime.py"))?,
            "def deploy(ctx):\n    custom(ctx)\n"
        );

        let deploy_mode = fs::metadata(infra_dir.join("deployment/build/01_build.sh"))?.permissions().mode() & 0o777;
        assert_eq!(deploy_mode, 0o755);

        Ok(())
    }

    #[test]
    fn refresh_local_infrastructure_refuses_managed_conflicts() -> Result<()> {
        let temp = TempDir::new()?;
        let source_dir = temp.path().join("source");
        let project_root = temp.path().join("project");
        let core_file = project_root.join("infra/provision/core/runtime.py");
        write(&source_dir.join("crates/bonesinfra/python/src/bonesinfra/frameworks/custom/runtime.py"), "managed")?;
        write(&project_root.join(".env"), "TEMPLATE=custom\n")?;
        write(&core_file, "locally changed")?;

        let error = match refresh_local_infrastructure(&source_dir, &project_root) {
            Ok(()) => anyhow::bail!("conflict should be rejected"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("Managed infrastructure conflict"));
        assert_eq!(fs::read_to_string(core_file)?, "locally changed");
        Ok(())
    }

    fn write(path: &Path, content: &str) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
        Ok(())
    }
}
