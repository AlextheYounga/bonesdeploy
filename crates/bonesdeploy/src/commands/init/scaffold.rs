use std::fs;
use std::os::unix::fs::{self as unix_fs, PermissionsExt};
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use shared::config::default_deploy_user;
use shared::paths;

use super::FrameworkSelection;
use crate::config;
use crate::frameworks;
use crate::infra::assets::{frameworks as framework_assets, kit};
use crate::infra::git;
use shared::env_build;

const PRE_PUSH_SCRIPT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/hooks/pre-push"));

pub(super) fn materialize_fresh_bones(
    bones_dir: &Path,
    had_bones_entry: bool,
    cfg: &mut config::Bones,
    framework: FrameworkSelection,
) -> Result<()> {
    let config_dir = config::bones_config_dir(&cfg.project_name);

    if config_dir.exists() && !config_dir.is_dir() {
        fs::remove_file(&config_dir)
            .with_context(|| format!("Stale file at {}, cannot create directory", config_dir.display()))?;
    }
    fs::create_dir_all(&config_dir)?;
    kit::scaffold(&config_dir)?;

    if had_bones_entry {
        fs::remove_file(bones_dir)
            .with_context(|| format!("Failed to remove stale {} symlink", bones_dir.display()))?;
    }
    unix_fs::symlink(&config_dir, bones_dir)?;

    setup_bones_git_repo(bones_dir, cfg)?;

    cfg.framework = serde_json::from_value(serde_json::Value::Object(framework.config.clone()))?;

    if let Some(template_name) = framework.template {
        framework_assets::scaffold_framework_env_build(&template_name, Path::new("."), &cfg.framework)?;
        framework_assets::scaffold_framework_deployment(&template_name, bones_dir)?;
        frameworks::configure(&template_name, cfg);
        println!("Runtime template: {template_name}");
    } else {
        println!("Runtime template: custom");
    }

    Ok(())
}

pub(super) fn update_gitignore() -> Result<()> {
    let gitignore = Path::new(paths::GITIGNORE_FILE);
    let entries = [paths::LOCAL_BONES_DIR, "!.env.build"];

    if gitignore.exists() {
        let content = fs::read_to_string(gitignore)?;
        let missing = entries
            .iter()
            .filter(|entry| !content.lines().any(|line| line.trim() == **entry))
            .copied()
            .collect::<Vec<_>>();
        if missing.is_empty() {
            return Ok(());
        }
        let separator = if content.ends_with('\n') { "" } else { "\n" };
        let additions = missing.join("\n");
        fs::write(gitignore, format!("{content}{separator}{additions}\n"))?;
    } else {
        fs::write(gitignore, format!("{}\n", entries.join("\n")))?;
    }

    Ok(())
}

pub(super) fn ensure_config_gitignore() -> Result<()> {
    let gitignore = paths::bones_config_root().join(paths::GITIGNORE_FILE);
    let project_entry = format!("{}/", paths::BONES_CONFIG_PROJECTS_DIR);

    if gitignore.exists() {
        let content = fs::read_to_string(&gitignore)?;
        let mut missing = Vec::new();
        for entry in [&project_entry] {
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
        let content = format!("{project_entry}\n");
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

fn setup_bones_git_repo(bones_dir: &Path, cfg: &config::Bones) -> Result<()> {
    let gitignore = bones_dir.join(paths::GITIGNORE_FILE);
    fs::write(&gitignore, paths::BONES_GITIGNORE_CONTENT)
        .with_context(|| format!("Failed to write {}", gitignore.display()))?;

    let output = Command::new("git")
        .args(["-C"])
        .arg(bones_dir)
        .args(["init", "--initial-branch", "master"])
        .output()
        .context("Failed to init git repo in .bones")?;
    if !output.status.success() {
        bail!("git init failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    let repo_path = paths::default_bones_repo_path_for(&cfg.project_name);
    let remote_url = if cfg.port == "22" {
        format!("{}@{}:{repo_path}", default_deploy_user(), cfg.host)
    } else {
        format!("ssh://{}@{}:{}{repo_path}", default_deploy_user(), cfg.host, cfg.port)
    };

    let output = Command::new("git")
        .args(["-C"])
        .arg(bones_dir)
        .args(["remote", "add", "origin", &remote_url])
        .output()
        .context("Failed to add remote to .bones repo")?;
    if !output.status.success() {
        bail!("Failed to add remote: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

pub(super) fn ensure_env_build() -> Result<()> {
    let env_build_path = Path::new(paths::ENV_BUILD_FILE);
    if env_build_path.exists() {
        return Ok(());
    }
    fs::write(env_build_path, env_build::default_content())
        .with_context(|| format!("Failed to write {}", env_build_path.display()))?;
    Ok(())
}
