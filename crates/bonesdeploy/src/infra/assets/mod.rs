pub mod frameworks;
pub mod kit;
pub mod skill;

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use bonesdeploy_core::paths;

use crate::platform;

fn write_asset(bones_dir: &Path, relative_path: &str, bytes: &[u8]) -> Result<()> {
    let dest = bones_dir.join(relative_path);

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    fs::write(&dest, bytes).with_context(|| format!("Failed to write {}", dest.display()))?;

    if relative_path.starts_with(paths::KIT_DEPLOYMENT_DIR) {
        #[cfg(unix)]
        platform::set_mode(&dest, 0o755)?;
        #[cfg(not(unix))]
        platform::set_mode(&dest, 0o755);
    }

    Ok(())
}
