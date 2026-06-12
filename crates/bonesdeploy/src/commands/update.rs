use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use console::style;
use tempfile::TempDir;

use crate::commands::update_release;
use crate::config;

const SOURCE_REPO_URL: &str = "https://github.com/AlextheYounga/bonesdeploy.git";
const SOURCE_BRANCH: &str = "master";

#[derive(Clone, Copy)]
pub struct UpdateOptions {
    pub skip_local: bool,
    pub skip_remote: bool,
}

pub async fn run(options: UpdateOptions) -> Result<()> {
    println!("{}", style("bonesdeploy update").bold());

    let current_local = update_release::current_local_version();
    let current_remote = update_release::current_remote_version();

    println!("Current local version: {}", style(&current_local).cyan());
    println!("Current remote version: {}", style(&current_remote).cyan());

    if options.skip_local && options.skip_remote {
        println!("{} Nothing to update.", style("Done!").green());
        return Ok(());
    }

    println!("Source branch: {}", style(SOURCE_BRANCH).cyan());

    let temp_dir = TempDir::new().context("Failed to create temp directory")?;
    let temp_path = temp_dir.path();

    println!("Checking master version from {}...", style(SOURCE_REPO_URL).cyan());
    let source_dir = clone_master_source(temp_path)?;
    let master_versions = read_master_versions(&source_dir)?;
    println!("Master bonesdeploy version: {}", style(&master_versions.bonesdeploy).cyan());
    println!("Master bonesremote version: {}", style(&master_versions.bonesremote).cyan());

    if !options.skip_local {
        if current_local == master_versions.bonesdeploy {
            println!("{} Local bonesdeploy is already current.", style("Done!").green());
        } else {
            println!("{}", style("Updating local bonesdeploy...").cyan());
            update_release::update_local_from_source(SOURCE_REPO_URL)?;
            println!("{} Local update complete.", style("Done!").green());
        }

        refresh_local_bones_from_source(&source_dir, Path::new(config::Constants::BONES_DIR))?;
    }

    if !options.skip_remote {
        if current_remote == master_versions.bonesremote {
            println!("{} Remote bonesremote is already current.", style("Done!").green());
        } else {
            println!("{}", style("Updating remote bonesremote...").cyan());
            update_release::update_remote_from_source(SOURCE_REPO_URL, &master_versions.bonesremote).await?;
            println!("{} Remote update complete.", style("Done!").green());
        }
    }

    println!("\n{} All updates complete.", style("Done!").green());

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
    let mut in_package_section = false;

    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed == "[package]" {
            in_package_section = true;
            continue;
        }
        if in_package_section && trimmed.starts_with('[') {
            break;
        }
        if in_package_section && let Some(version) = parse_version_line(trimmed) {
            return Ok(version);
        }
    }

    bail!("missing [package] version")
}

fn parse_version_line(line: &str) -> Option<String> {
    let value = line.strip_prefix("version")?.trim_start().strip_prefix('=')?.trim();
    let version = value.strip_prefix('"')?.strip_suffix('"')?;
    (!version.is_empty()).then(|| version.to_string())
}

fn refresh_local_bones_from_source(source_dir: &Path, bones_dir: &Path) -> Result<()> {
    if !bones_dir.exists() {
        return Ok(());
    }

    println!("{}", style("Refreshing local .bones scaffold...").cyan());

    let kit_root = source_dir.join("crates/bonesdeploy/embeds/kit");
    sync_tree(&kit_root.join("hooks"), &bones_dir.join("hooks"), true)?;
    sync_tree(&kit_root.join("deployment"), &bones_dir.join("deployment"), true)?;

    let infra_root = source_dir.join("infra");
    sync_infra_tree(&infra_root, &bones_dir.join("infra"), Path::new(""))?;

    if let Some(template_name) = current_runtime_template(bones_dir)? {
        let runtime_root = source_dir.join(format!("crates/bonesdeploy/embeds/runtimes/{template_name}/infra"));
        if runtime_root.is_dir() {
            sync_tree(&runtime_root, &bones_dir.join("infra"), false)?;
        }
    }

    println!("{} Local .bones scaffold refreshed.", style("Done!").green());
    Ok(())
}

fn current_runtime_template(bones_dir: &Path) -> Result<Option<String>> {
    let runtime_yaml = bones_dir.join("runtime.yaml");
    if !runtime_yaml.is_file() {
        return Ok(None);
    }

    let runtime = config::load_runtime(&runtime_yaml)?;
    Ok(runtime.get("template").and_then(serde_json::Value::as_str).map(str::to_string))
}

fn sync_infra_tree(source_root: &Path, dest_root: &Path, relative: &Path) -> Result<()> {
    for entry in fs::read_dir(source_root).with_context(|| format!("Failed to read {}", source_root.display()))? {
        let entry = entry.with_context(|| format!("Failed to read entry in {}", source_root.display()))?;
        let file_type =
            entry.file_type().with_context(|| format!("Failed to read file type for {}", entry.path().display()))?;
        let name = entry.file_name();
        let next_relative = relative.join(&name);

        if should_skip_infra_path(&next_relative, file_type.is_dir()) {
            continue;
        }

        let source_path = entry.path();
        let dest_path = dest_root.join(&next_relative);

        if file_type.is_dir() {
            fs::create_dir_all(&dest_path).with_context(|| format!("Failed to create {}", dest_path.display()))?;
            sync_infra_tree(&source_path, dest_root, &next_relative)?;
            continue;
        }

        copy_file(&source_path, &dest_path, false)?;
    }

    Ok(())
}

fn should_skip_infra_path(relative: &Path, is_dir: bool) -> bool {
    let Some(first) = relative.components().next().map(std::path::Component::as_os_str) else {
        return false;
    };

    if is_dir {
        return first == "__pycache__" || first == ".venv";
    }

    first == ".gitignore"
        || first == "README.md"
        || first == ".python-version"
        || first == "pyproject.toml"
        || first == "uv.lock"
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

        write(&source_dir.join("crates/bonesdeploy/embeds/kit/hooks/pre-push"), "new hook")?;
        write(&source_dir.join("crates/bonesdeploy/embeds/kit/deployment/01_build.sh"), "new deploy")?;
        write(&source_dir.join("infra/runtime.py"), "new runtime")?;
        write(&source_dir.join("infra/README.md"), "skip me")?;
        write(&source_dir.join("crates/bonesdeploy/embeds/runtimes/laravel/infra/operations.py"), "laravel ops")?;

        write(&bones_dir.join("bones.yaml"), "keep: config\n")?;
        write(&bones_dir.join("runtime.yaml"), "template: laravel\n")?;
        write(&bones_dir.join("infra/runtime.py"), "old runtime")?;

        refresh_local_bones_from_source(&source_dir, &bones_dir)?;

        assert_eq!(fs::read_to_string(bones_dir.join("bones.yaml"))?, "keep: config\n");
        assert_eq!(fs::read_to_string(bones_dir.join("runtime.yaml"))?, "template: laravel\n");
        assert_eq!(fs::read_to_string(bones_dir.join("hooks/pre-push"))?, "new hook");
        assert_eq!(fs::read_to_string(bones_dir.join("deployment/01_build.sh"))?, "new deploy");
        assert_eq!(fs::read_to_string(bones_dir.join("infra/runtime.py"))?, "new runtime");
        assert_eq!(fs::read_to_string(bones_dir.join("infra/operations.py"))?, "laravel ops");
        assert!(!bones_dir.join("infra/README.md").exists());

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
