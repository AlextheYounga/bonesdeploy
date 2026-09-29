use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use bonesdeploy_core::build_contract;
use bonesdeploy_core::config::{build_group_for, build_timeout_seconds, build_user_for};
use bonesdeploy_core::paths;

use super::build_user::BuildScriptEnv;
use super::container::BuildContainer;
use super::ownership;

pub fn run(snapshot: &super::super::DeploymentSnapshot, context: &Path) -> Result<()> {
    if !context.is_dir() {
        bail!("Build context does not exist: {}", context.display());
    }

    let cfg = &snapshot.config;
    let build_user = build_user_for(&cfg.project_name);
    let build_group = build_group_for(&cfg.project_name);
    ownership::chown_tree_to_user(context, &build_user, &build_group)?;

    let scripts_dir = snapshot.deployment_dir.join(paths::DEPLOYMENT_BUILD_DIR);
    if !scripts_dir.is_dir() {
        println!(
            "No deployment scripts at {}; running build steps directly on the exported source tree.",
            scripts_dir.display()
        );
        return Ok(());
    }

    let scripts = build_contract::numbered_scripts(&scripts_dir)?;
    if scripts.is_empty() {
        println!("No deployment scripts found at {}; skipping build.", scripts_dir.display());
        return Ok(());
    }

    let build_env_vars = build_contract::environment(cfg, context)?;
    let deployment_dir = scripts_dir.parent().context("Build scripts directory has no deployment parent")?;
    let build_cache_dir = paths::bonesdeploy_user_cache(&build_user);

    let build_env = BuildScriptEnv {
        project_name: &cfg.project_name,
        build_user: &build_user,
        build_group: &build_group,
        web_root: &cfg.runtime.web_root,
        deployment_dir,
        build_cache_dir: &build_cache_dir,
        build_env_vars: &build_env_vars,
        script_timeout_seconds: build_timeout_seconds(cfg),
    };
    let mut container = BuildContainer::start(context, &build_env)?;

    let logs_dir = paths::bonesremote_site_logs(&snapshot.site);
    fs::create_dir_all(&logs_dir).with_context(|| format!("Failed to create logs directory {}", logs_dir.display()))?;

    for script in scripts {
        let script_name = script.file_name().and_then(|name| name.to_str()).unwrap_or("<unknown>");
        println!("Running build script {script_name}...");

        let status = container
            .run_script(&script, &logs_dir.join(format!("{script_name}.log")))
            .with_context(|| format!("Failed to execute build script {}", script.display()))?;

        if !status.success() {
            bail!("Build script {script_name} exited with status {status}");
        }
    }

    container.remove()?;

    Ok(())
}
