use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use bonesdeploy_core::paths;

use crate::git;

fn resolved_tmp_root(site: &str) -> PathBuf {
    PathBuf::from(paths::default_project_root_for(site)).join(paths::TMP_BUILDS_DIR)
}

pub fn run(snapshot: &super::DeploymentSnapshot, context_dir: &Path) -> Result<()> {
    let archive_output = git::archive(&snapshot.repo_path, &snapshot.revision)?;
    let mut archive = archive_output.as_slice();

    let mut tar = Command::new("tar")
        .args(["-xf", "-", "-C"])
        .arg(&context_dir)
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to start tar extraction into {}", context_dir.display()))?;

    let mut tar_stdin = tar.stdin.take().context("tar stdin was not piped")?;
    io::copy(&mut archive, &mut tar_stdin).context("Failed to stream git archive into tar")?;
    drop(tar_stdin);

    let tar_output = tar.wait_with_output().context("Failed to finish tar extraction")?;

    if !tar_output.status.success() {
        bail!(
            "Failed to extract source archive into build context {}\n{}",
            context_dir.display(),
            String::from_utf8_lossy(&tar_output.stderr)
        );
    }

    let deployment_dir = context_dir.join(paths::LOCAL_INFRA_DIR).join(paths::DEPLOYMENT_DIR);
    if deployment_dir.is_dir() {
        println!("Using deployment files from {}", deployment_dir.display());
    }
    println!("Exported source for {} into {}", snapshot.revision, context_dir.display());
    Ok(())
}

pub fn ensure_build_context(snapshot: &super::DeploymentSnapshot, release: &str) -> Result<PathBuf> {
    let root = resolved_tmp_root(&snapshot.site);
    if !root.is_dir() {
        bail!("Build workspace {} is missing. Run 'bonesdeploy site setup' to provision it.", root.display());
    }
    let context = root.join(format!("build-{release}"));
    fs::create_dir(&context).with_context(|| format!("Failed to create build context {}", context.display()))?;
    Ok(context)
}

pub fn cleanup_build_context(site: &str, context: &Path) -> Result<()> {
    if context.exists() {
        fs::remove_dir_all(context).with_context(|| format!("Failed to remove build context {}", context.display()))?;
    }
    let _ = site;
    Ok(())
}
