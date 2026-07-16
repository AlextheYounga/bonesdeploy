use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use shared::paths;

pub(super) fn refresh_local_bones_from_source(source_dir: &Path, bones_dir: &Path) -> Result<()> {
    if !bones_dir.exists() {
        return Ok(());
    }

    sync_tree(&deployment_source_root(source_dir, bones_dir)?, &bones_dir.join("deployment"), true)?;

    Ok(())
}

fn deployment_source_root(source_dir: &Path, bones_dir: &Path) -> Result<PathBuf> {
    let bones_toml = bones_dir.join(paths::BONES_TOML);
    let Some(template) = selected_runtime_template(&bones_toml)? else {
        return Ok(source_dir.join("crates/bonesdeploy/kit/deployment"));
    };

    let runtime_deployment = source_dir.join("crates/bonesdeploy/runtimes").join(template).join("deployment");
    Ok(if runtime_deployment.is_dir() {
        runtime_deployment
    } else {
        source_dir.join("crates/bonesdeploy/kit/deployment")
    })
}

fn selected_runtime_template(runtime_toml: &Path) -> Result<Option<String>> {
    let content =
        fs::read_to_string(runtime_toml).with_context(|| format!("Failed to read {}", runtime_toml.display()))?;
    let value: toml::Value =
        toml::from_str(&content).with_context(|| format!("Failed to parse {}", runtime_toml.display()))?;
    Ok(value.get("runtime").and_then(|runtime| runtime.get("template")).and_then(toml::Value::as_str).map(String::from))
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

    use super::refresh_local_bones_from_source;
    use anyhow::Result;
    use tempfile::TempDir;

    #[test]
    fn refresh_local_bones_updates_scaffold_without_touching_configs() -> Result<()> {
        let temp = TempDir::new()?;
        let source_dir = temp.path().join("source");
        let bones_dir = temp.path().join(".bones");

        write(&source_dir.join("crates/bonesdeploy/kit/deployment/build/01_build.sh"), "generic deploy")?;
        write(&source_dir.join("crates/bonesdeploy/runtimes/laravel/deployment/build/01_build.sh"), "laravel deploy")?;

        write(&bones_dir.join("bones.toml"), "[runtime]\ntemplate = 'laravel'\n")?;

        refresh_local_bones_from_source(&source_dir, &bones_dir)?;

        assert_eq!(fs::read_to_string(bones_dir.join("bones.toml"))?, "[runtime]\ntemplate = 'laravel'\n");
        assert_eq!(fs::read_to_string(bones_dir.join("deployment/build/01_build.sh"))?, "laravel deploy");

        let deploy_mode = fs::metadata(bones_dir.join("deployment/build/01_build.sh"))?.permissions().mode() & 0o777;
        assert_eq!(deploy_mode, 0o755);

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
