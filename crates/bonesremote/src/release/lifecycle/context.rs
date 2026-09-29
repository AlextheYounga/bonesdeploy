use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use bonesdeploy_core::paths;

use crate::git;

fn resolved_tmp_root(site: &str) -> PathBuf {
    PathBuf::from(paths::default_project_root_for(site)).join(paths::TMP_BUILDS_DIR)
}

pub fn create(snapshot: &super::DeploymentSnapshot) -> Result<PathBuf> {
    let root = resolved_tmp_root(&snapshot.site);
    fs::create_dir_all(&root)
        .with_context(|| format!("Failed to create temporary artifact root: {}", root.display()))?;

    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
    let context = root.join(format!("build-{}-{nanos}", snapshot.site));
    fs::create_dir_all(&context).with_context(|| format!("Failed to create artifact context {}", context.display()))?;
    Ok(context)
}

pub fn cleanup(site: &str, context: &Path) -> Result<()> {
    if context.exists() {
        fs::remove_dir_all(context)
            .with_context(|| format!("Failed to remove artifact context {}", context.display()))?;
    }
    let root = context.parent().map_or_else(|| resolved_tmp_root(site), Path::to_path_buf);
    if root.exists() && fs::read_dir(&root)?.next().is_none() {
        fs::remove_dir(&root).ok();
    }
    Ok(())
}

pub fn materialize_repository(snapshot: &super::DeploymentSnapshot, context: &Path) -> Result<()> {
    let archive = git::archive(&snapshot.repo_path, &snapshot.revision)?;
    let mut extract = Command::new("tar")
        .args(["-xf", "-", "-C"])
        .arg(context)
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to start Compose source extraction into {}", context.display()))?;
    let mut input = extract.stdin.take().context("tar stdin was not piped")?;
    input.write_all(&archive).context("Failed to stream Compose source archive into tar")?;
    drop(input);
    let output = extract.wait_with_output().context("Failed to finish Compose source extraction")?;
    if !output.status.success() {
        anyhow::bail!(
            "Failed to extract Compose source archive into {}\n{}",
            context.display(),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}
