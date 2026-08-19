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

    let selected_framework = framework
        .template
        .as_deref()
        .map(frameworks::Framework::parse)
        .transpose()?
        .unwrap_or(frameworks::Framework::Custom);
    if selected_framework != frameworks::Framework::Custom {
        framework_assets::scaffold_framework_env_build(&selected_framework.to_string(), Path::new("."), &cfg.runtime)?;
        selected_framework.configure(cfg);
    }
    println!("Framework template: {selected_framework}");

    framework_assets::scaffold_framework_project(&selected_framework.to_string(), infra_dir)?;
    bonesinfra::materialize_project_core(Path::new("."))?;
    scaffold_custom_provisioning(infra_dir)?;
    fs::create_dir_all(paths::LOCAL_INFRA_SECRETS_DIR)
        .with_context(|| format!("Failed to create {}", paths::LOCAL_INFRA_SECRETS_DIR))?;
    Ok(())
}

fn scaffold_custom_provisioning(infra_dir: &Path) -> Result<()> {
    let custom = infra_dir.join("provision/custom");
    fs::create_dir_all(&custom).with_context(|| format!("Failed to create {}", custom.display()))?;
    fs::write(custom.join("__init__.py"), "\"\"\"Project-owned provisioning.\"\"\"\n")?;
    fs::write(custom.join("runtime.py"), "def deploy(_ctx):\n    pass\n")?;
    fs::write(
        custom.join("manifest.py"),
        "def artifacts(_ctx):\n    return []\n\n\ndef services(_ctx):\n    return []\n\n\ndef mode(_ctx):\n    return None\n",
    )?;
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
