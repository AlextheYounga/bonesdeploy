use std::fs;
use std::os::unix::fs::{self as unix_fs, PermissionsExt};
use std::path::Path;

use anyhow::{Context, Result};
use shared::config::default_deploy_user;
use shared::paths;

use super::RuntimeSelection;
use crate::config;
use crate::infra::embedded;
use crate::infra::git;

const PRE_PUSH_SCRIPT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/hooks/pre-push"));

pub(super) fn materialize_fresh_bones(
    bones_dir: &Path,
    had_bones_entry: bool,
    cfg: &config::Bones,
    runtime: RuntimeSelection,
) -> Result<()> {
    let config_dir = config::bones_config_dir(&cfg.project_name);

    if config_dir.exists() && !config_dir.is_dir() {
        fs::remove_file(&config_dir)
            .with_context(|| format!("Stale file at {}, cannot create directory", config_dir.display()))?;
    }
    fs::create_dir_all(&config_dir)?;
    embedded::scaffold(&config_dir)?;

    if had_bones_entry {
        fs::remove_file(bones_dir)
            .with_context(|| format!("Failed to remove stale {} symlink", bones_dir.display()))?;
    }
    unix_fs::symlink(&config_dir, bones_dir)?;

    let runtime_toml = Path::new(paths::LOCAL_BONES_RUNTIME_TOML);
    config::save_runtime(&runtime.config, runtime_toml)?;

    if let Some(template_name) = runtime.template {
        embedded::scaffold_runtime_deployment(&template_name, bones_dir)?;
        embedded::scaffold_runtime_secrets(&template_name, bones_dir)?;
        println!("Runtime template: {template_name}");
    } else {
        println!("Runtime template: custom");
    }

    Ok(())
}

pub(super) fn update_gitignore() -> Result<()> {
    let gitignore = Path::new(".gitignore");
    let entry = paths::LOCAL_BONES_DIR;

    if gitignore.exists() {
        let content = fs::read_to_string(gitignore)?;
        if content.lines().any(|line| line.trim() == entry) {
            return Ok(());
        }
        let separator = if content.ends_with('\n') { "" } else { "\n" };
        fs::write(gitignore, format!("{content}{separator}{entry}\n"))?;
    } else {
        fs::write(gitignore, format!("{entry}\n"))?;
    }

    Ok(())
}

pub(super) fn ensure_config_gitignore(project_name: &str) -> Result<()> {
    let gitignore = paths::bones_config_root().join(".gitignore");
    let project_entry = format!("{project_name}.bones");

    if gitignore.exists() {
        let content = fs::read_to_string(&gitignore)?;
        let mut missing = Vec::new();
        for entry in [paths::BONES_CONFIG_LIB_DIR, &project_entry] {
            if !content.lines().any(|line| line.trim() == entry) {
                missing.push(entry);
            }
        }
        if missing.is_empty() {
            return Ok(());
        }
        let separator = if content.ends_with('\n') { "" } else { "\n" };
        let mut append = String::new();
        for entry in &missing {
            append.push_str(entry);
            append.push('\n');
        }
        fs::write(&gitignore, format!("{content}{separator}{append}"))?;
    } else {
        let mut content = String::new();
        for entry in [paths::BONES_CONFIG_LIB_DIR, &project_entry] {
            content.push_str(entry);
            content.push('\n');
        }
        fs::write(&gitignore, content)?;
    }

    Ok(())
}

pub(super) fn install_pre_push_guard() -> Result<()> {
    let hooks_dir = Path::new(paths::GIT_HOOKS_DIR);
    fs::create_dir_all(hooks_dir)?;

    let guard = hooks_dir.join(paths::PRE_PUSH_HOOK_NAME);

    if fs::symlink_metadata(&guard).is_ok() {
        fs::remove_file(&guard).with_context(|| format!("Failed to remove existing {}", guard.display()))?;
    }

    fs::write(&guard, PRE_PUSH_SCRIPT).with_context(|| format!("Failed to write {}", guard.display()))?;
    let mut perms = fs::metadata(&guard).with_context(|| format!("Failed to stat {}", guard.display()))?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&guard, perms).with_context(|| format!("Failed to chmod {}", guard.display()))?;

    Ok(())
}

pub(super) fn ensure_local_remote(cfg: &config::Bones) -> Result<()> {
    if git::remote_exists(&cfg.remote_name)? {
        return Ok(());
    }

    let remote_url = format!("{}@{}:{}", default_deploy_user(), cfg.host, cfg.repo_path);
    git::add_remote(&cfg.remote_name, &remote_url)?;
    Ok(())
}
