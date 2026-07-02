use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use shared::config::{self, Runtime, build_group_for, build_user_for, load_runtime};
use shared::paths;
use shared::paths::default_web_root;

use super::ownership;
use crate::release::script_runner as deploy_output;

pub(super) fn run(site: &str, context: &Path, cfg: &config::Bones) -> Result<()> {
    if !context.is_dir() {
        bail!("Build context does not exist: {}", context.display());
    }

    let build_user = build_user_for(&cfg.project_name);
    let build_group = build_group_for(&cfg.project_name);
    ownership::chown_tree_to_user(context, &build_user, &build_group)?;

    let scripts_dir = paths::bonesremote_site_root(site).join(paths::DEPLOYMENT_DIR).join(paths::DEPLOYMENT_BUILD_DIR);
    if !scripts_dir.is_dir() {
        println!(
            "No deployment scripts at {}; running build steps directly on the exported source tree.",
            scripts_dir.display()
        );
        return Ok(());
    }

    let scripts = list_scripts(&scripts_dir)?;
    if scripts.is_empty() {
        println!("No deployment scripts found at {}; skipping build.", scripts_dir.display());
        return Ok(());
    }

    let runtime = load_runtime_or_default(site);

    let build_env = deploy_output::BuildScriptEnv {
        project_name: &cfg.project_name,
        build_user: &build_user,
        build_uid: ownership::user_uid(&build_user)?,
        web_root: &runtime.web_root,
    };
    let mut container = deploy_output::BuildContainer::start(context, &build_env)?;

    for script in scripts {
        let script_name = script.file_name().and_then(|name| name.to_str()).unwrap_or("<unknown>");
        println!("Running build script {script_name}...");

        let status = container
            .run_script(&script, &context.join(format!("{script_name}.log")))
            .with_context(|| format!("Failed to execute build script {}", script.display()))?;

        if !status.success() {
            bail!("Build script {script_name} exited with status {status}");
        }
    }

    container.remove()?;

    Ok(())
}

fn load_runtime_or_default(site: &str) -> Runtime {
    load_runtime(&paths::bonesremote_site_root(site)).unwrap_or_else(|_| Runtime {
        web_root: default_web_root(),
        runtime_user: String::new(),
        runtime_group: String::new(),
        release_group: String::new(),
        shared: config::Shared::default(),
    })
}

fn list_scripts(scripts_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut scripts = Vec::new();
    for entry in
        fs::read_dir(scripts_dir).with_context(|| format!("Failed to read scripts dir: {}", scripts_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            scripts.push(path);
        }
    }
    scripts.sort();
    Ok(scripts)
}
