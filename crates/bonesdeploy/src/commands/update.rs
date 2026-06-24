use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use tempfile::TempDir;

use super::update_release;
use shared::paths;

const SOURCE_REPO_URL: &str = "https://github.com/AlextheYounga/bonesdeploy.git";
const SOURCE_BRANCH: &str = "master";

#[derive(Clone, Copy)]
pub struct Options {
    pub skip_local: bool,
    pub skip_remote: bool,
}

pub async fn run(options: Options) -> Result<()> {
    println!("Checking for updates...");
    let current_local = update_release::current_local_version();
    let current_remote = update_release::current_remote_version();

    if options.skip_local && options.skip_remote {
        println!("Already up to date.");
        return Ok(());
    }

    let temp_dir = TempDir::new().context("Failed to create temp directory")?;
    let temp_path = temp_dir.path();

    let source_dir = clone_master_source(temp_path)?;
    let master_versions = read_master_versions(&source_dir)?;

    let mut updated = false;

    if !options.skip_local {
        if current_local != master_versions.bonesdeploy {
            println!("Updating bonesdeploy...");
            update_release::update_local_from_source(SOURCE_REPO_URL)?;
            updated = true;
        }

        refresh_local_bones_from_source(&source_dir, Path::new(paths::LOCAL_BONES_DIR))?;
    }

    if !options.skip_remote && current_remote != master_versions.bonesremote {
        println!("Updating bonesremote...");
        update_release::update_remote_from_source(SOURCE_REPO_URL, &master_versions.bonesremote).await?;
        updated = true;
    }

    if updated {
        println!("Update complete.");
    } else {
        println!("Already up to date.");
    }

    Ok(())
}

fn clone_master_source(temp_path: &Path) -> Result<PathBuf> {
    let source_dir = temp_path.join("source");

    let clone_status = Command::new("git")
        .args(["clone", "--depth", "1", "--branch", SOURCE_BRANCH, SOURCE_REPO_URL])
        .arg(&source_dir)
        .status()
        .context("Failed to clone bonesdeploy repository")?;

    if !clone_status.success() {
        bail!("Failed to clone {SOURCE_REPO_URL} branch {SOURCE_BRANCH}");
    }

    Ok(source_dir)
}

struct MasterVersions {
    bonesdeploy: String,
    bonesremote: String,
}

fn read_master_versions(source_dir: &Path) -> Result<MasterVersions> {
    let bonesdeploy = read_package_version(&source_dir.join("crates/bonesdeploy/Cargo.toml"))?;
    let bonesremote = read_package_version(&source_dir.join("crates/bonesremote/Cargo.toml"))?;

    Ok(MasterVersions { bonesdeploy, bonesremote })
}

fn read_package_version(manifest: &Path) -> Result<String> {
    let content = fs::read_to_string(manifest).with_context(|| format!("Failed to read {}", manifest.display()))?;
    parse_package_version(&content)
        .with_context(|| format!("Failed to parse package version from {}", manifest.display()))
}

fn parse_package_version(manifest: &str) -> Result<String> {
    let value: toml::Value = toml::from_str(manifest)?;
    value
        .get("package")
        .and_then(|package| package.get("version"))
        .and_then(toml::Value::as_str)
        .filter(|version| !version.is_empty())
        .map(String::from)
        .ok_or_else(|| anyhow::anyhow!("missing [package] version"))
}

fn refresh_local_bones_from_source(source_dir: &Path, bones_dir: &Path) -> Result<()> {
    if !bones_dir.exists() {
        return Ok(());
    }

    let kit_root = source_dir.join("crates/bonesdeploy/kit");
    sync_tree(&kit_root.join("hooks"), &bones_dir.join("hooks"), true)?;
    sync_tree(&deployment_source_root(source_dir, bones_dir), &bones_dir.join("deployment"), true)?;

    Ok(())
}

fn deployment_source_root(source_dir: &Path, bones_dir: &Path) -> PathBuf {
    let runtime_toml = bones_dir.join("runtime.toml");
    let Some(template) = selected_runtime_template(&runtime_toml) else {
        return source_dir.join("crates/bonesdeploy/kit/deployment");
    };

    let runtime_deployment = source_dir.join("crates/bonesdeploy/runtimes").join(template).join("deployment");
    if runtime_deployment.is_dir() { runtime_deployment } else { source_dir.join("crates/bonesdeploy/kit/deployment") }
}

fn selected_runtime_template(runtime_toml: &Path) -> Option<String> {
    let content = fs::read_to_string(runtime_toml).ok()?;
    let value: toml::Value = toml::from_str(&content).ok()?;
    value.get("template")?.as_str().map(String::from)
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

    use anyhow::Result;
    use tempfile::TempDir;

    use super::{SOURCE_BRANCH, SOURCE_REPO_URL, parse_package_version, refresh_local_bones_from_source};

    /// Verifies the update source repository and branch constants are set to the canonical values.
    #[test]
    fn update_uses_master_branch_source_repository() {
        assert_eq!(SOURCE_REPO_URL, "https://github.com/AlextheYounga/bonesdeploy.git");
        assert_eq!(SOURCE_BRANCH, "master");
    }

    /// Extracts the package version from the `[package]` section of a Cargo manifest.
    #[test]
    fn parses_package_version_from_manifest_package_section() -> anyhow::Result<()> {
        let manifest = r#"
[package]
name = "bonesdeploy"
version = "0.2.8"
edition = "2024"

[dependencies]
version = "not-this"
"#;

        assert_eq!(parse_package_version(manifest)?, "0.2.8");
        Ok(())
    }

    /// Returns an error when the manifest has no `[package]` section with a version field.
    #[test]
    fn rejects_manifest_without_package_version() {
        let result = parse_package_version("[dependencies]\nversion = \"0.2.8\"\n");
        assert!(result.is_err());
    }

    /// Refreshes .bones scaffold assets from the cloned source tree without overwriting local config files.
    #[test]
    fn refresh_local_bones_updates_scaffold_without_touching_configs() -> Result<()> {
        let temp = TempDir::new()?;
        let source_dir = temp.path().join("source");
        let bones_dir = temp.path().join(".bones");

        write(&source_dir.join("crates/bonesdeploy/kit/hooks/pre-push"), "new hook")?;
        write(&source_dir.join("crates/bonesdeploy/kit/deployment/01_build.sh"), "generic deploy")?;
        write(&source_dir.join("crates/bonesdeploy/runtimes/laravel/deployment/01_build.sh"), "laravel deploy")?;

        write(&bones_dir.join("bones.toml"), "keep = 'config'\n")?;
        write(&bones_dir.join("runtime.toml"), "template = 'laravel'\n")?;

        refresh_local_bones_from_source(&source_dir, &bones_dir)?;

        assert_eq!(fs::read_to_string(bones_dir.join("bones.toml"))?, "keep = 'config'\n");
        assert_eq!(fs::read_to_string(bones_dir.join("runtime.toml"))?, "template = 'laravel'\n");
        assert_eq!(fs::read_to_string(bones_dir.join("hooks/pre-push"))?, "new hook");
        assert_eq!(fs::read_to_string(bones_dir.join("deployment/01_build.sh"))?, "laravel deploy");

        let hook_mode = fs::metadata(bones_dir.join("hooks/pre-push"))?.permissions().mode() & 0o777;
        let deploy_mode = fs::metadata(bones_dir.join("deployment/01_build.sh"))?.permissions().mode() & 0o777;
        assert_eq!(hook_mode, 0o755);
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
