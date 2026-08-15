use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use bonesdeploy_core::config::default_deploy_user;
use bonesdeploy_core::paths;

use super::FrameworkSelection;
use crate::config;
use crate::frameworks;
use crate::infra::assets::frameworks as framework_assets;
use crate::infra::git;
use bonesdeploy_core::env_build;

pub(super) fn materialize_project(cfg: &mut config::Bones, framework: FrameworkSelection) -> Result<()> {
    cfg.runtime = serde_json::from_value(serde_json::Value::Object(framework.config))?;
    let infra_dir = Path::new(paths::LOCAL_INFRA_DIR);
    fs::create_dir_all(infra_dir)?;

    let framework_name = framework.template.as_deref().unwrap_or("custom");
    if let Some(template_name) = framework.template.as_deref() {
        framework_assets::scaffold_framework_env_build(template_name, Path::new("."), &cfg.runtime)?;
        frameworks::configure(template_name, cfg);
        println!("Framework template: {template_name}");
    } else {
        println!("Framework template: custom");
    }

    framework_assets::scaffold_framework_project(framework_name, infra_dir)?;
    bonesinfra::run(&["project", "materialize", "--env-file", paths::DOT_ENV, "--framework", framework_name])?;
    fs::create_dir_all(paths::LOCAL_INFRA_SECRETS_DIR)
        .with_context(|| format!("Failed to create {}", paths::LOCAL_INFRA_SECRETS_DIR))?;
    Ok(())
}

pub(super) fn update_gitignore() -> Result<()> {
    let gitignore = Path::new(paths::GITIGNORE_FILE);
    let entries = [paths::DOT_ENV, "!.env.build"];

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
        fs::write(gitignore, format!("{content}{separator}{}\n", missing.join("\n")))?;
    } else {
        fs::write(gitignore, format!("{}\n", entries.join("\n")))?;
    }

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

pub(super) fn ensure_env_build() -> Result<()> {
    let env_build_path = Path::new(paths::ENV_BUILD_FILE);
    if env_build_path.exists() {
        return Ok(());
    }
    fs::write(env_build_path, env_build::default_content())
        .with_context(|| format!("Failed to write {}", env_build_path.display()))?;
    Ok(())
}
