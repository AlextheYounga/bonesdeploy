use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::config;
use crate::privileges;
use crate::release_state;

use super::activate_release;
use super::drop_failed_release;

pub fn run(config_path: &str) -> Result<()> {
    privileges::ensure_not_root("bonesremote hooks deploy")?;

    let cfg = config::load(Path::new(config_path))?;
    let release_name = release_state::read_staged_release(&cfg)?;
    let runtime_path = release_state::release_dir(&cfg, &release_name);
    let build_root = release_state::build_root(&cfg);
    let deployment_dir = Path::new(&cfg.data.git_dir).join("bones").join("deployment");

    if !runtime_path.exists() {
        bail!("Staged runtime directory does not exist: {}", runtime_path.display());
    }

    if !build_root.exists() {
        bail!("Build workspace does not exist: {}", build_root.display());
    }

    let scripts = list_deployment_scripts(&deployment_dir)?;
    if scripts.is_empty() {
        println!("No deployment scripts found. Skipping deploy scripts.");
    } else {
        for script in scripts {
            let script_name = script.file_name().and_then(|name| name.to_str()).unwrap_or("<unknown>");
            println!("Running {script_name}...");

            let status = Command::new("bash")
                .arg(&script)
                .current_dir(&build_root)
                .status()
                .with_context(|| format!("Failed to execute deployment script {}", script.display()))?;

            if !status.success() {
                println!("Deployment script {script_name} failed.");
                drop_failed_release::run(config_path)
                    .with_context(|| "Failed to drop staged release after deployment script failure")?;
                bail!("Deployment script {script_name} failed with status {status}");
            }
        }

        println!("All deployment scripts completed.");
    }

    publish_runtime_tree(&build_root, &runtime_path)?;

    activate_release::run(config_path)
}

fn publish_runtime_tree(build_root: &Path, runtime_path: &Path) -> Result<()> {
    clear_directory(runtime_path)?;

    let copy_source = build_root.join(".");
    let status = Command::new("cp").arg("-a").arg(&copy_source).arg(runtime_path).status().with_context(|| {
        format!("Failed to copy build workspace {} to runtime tree {}", build_root.display(), runtime_path.display())
    })?;

    if !status.success() {
        bail!(
            "Failed to publish runtime tree from {} to {}: status {status}",
            build_root.display(),
            runtime_path.display()
        );
    }

    println!("Published runtime tree: {}", runtime_path.display());
    Ok(())
}

fn clear_directory(path: &Path) -> Result<()> {
    for entry in fs::read_dir(path).with_context(|| format!("Failed to read directory {}", path.display()))? {
        let entry = entry?;
        let entry_path = entry.path();
        let file_type = entry.file_type().with_context(|| format!("Failed to inspect {}", entry_path.display()))?;

        if file_type.is_dir() {
            fs::remove_dir_all(&entry_path)
                .with_context(|| format!("Failed to remove directory {}", entry_path.display()))?;
        } else {
            fs::remove_file(&entry_path).with_context(|| format!("Failed to remove {}", entry_path.display()))?;
        }
    }

    Ok(())
}

fn list_deployment_scripts(deployment_dir: &Path) -> Result<Vec<PathBuf>> {
    if !deployment_dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut scripts = Vec::new();
    for entry in fs::read_dir(deployment_dir)
        .with_context(|| format!("Failed to read deployment directory {}", deployment_dir.display()))?
    {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            scripts.push(entry.path());
        }
    }

    scripts.sort();
    Ok(scripts)
}
