use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use bonesdeploy_core::paths;

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
