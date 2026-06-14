use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use anyhow::{Context, Result};
use time::OffsetDateTime;
use time::format_description::FormatItem;
use time::macros::format_description;

use crate::config;
use crate::release_state;

pub fn run(config_path: &str) -> Result<()> {
    let cfg = config::load(Path::new(config_path))?;

    let project_root = Path::new(&cfg.data.project_root);
    let project_root_parent = project_root.parent().unwrap_or(project_root);
    let build_dir = project_root.join(config::Constants::BUILD_DIR);
    let build_root = release_state::build_root(&cfg);
    let releases_dir = release_state::releases_dir(&cfg);
    let shared_dir = release_state::shared_dir(&cfg);

    fs::create_dir_all(project_root)
        .with_context(|| format!("Failed to create project_root: {}", project_root.display()))?;
    ensure_non_owner_can_traverse(project_root_parent)
        .with_context(|| format!("Failed to set traverse permission on {}", project_root_parent.display()))?;
    fs::create_dir_all(&releases_dir)
        .with_context(|| format!("Failed to create releases dir: {}", releases_dir.display()))?;
    fs::create_dir_all(&build_dir).with_context(|| format!("Failed to create build dir: {}", build_dir.display()))?;
    fs::create_dir_all(&build_root)
        .with_context(|| format!("Failed to create build workspace: {}", build_root.display()))?;
    fs::create_dir_all(&shared_dir)
        .with_context(|| format!("Failed to create shared dir: {}", shared_dir.display()))?;

    let release_name = create_release_name()?;
    let staged_release_dir = release_state::release_dir(&cfg, &release_name);
    fs::create_dir_all(&staged_release_dir)
        .with_context(|| format!("Failed to create release dir: {}", staged_release_dir.display()))?;

    release_state::write_staged_release(&cfg, &release_name)?;

    println!("Staged release: {release_name}");
    Ok(())
}

fn ensure_non_owner_can_traverse(path: &Path) -> Result<()> {
    let metadata = fs::metadata(path).with_context(|| format!("Failed to read metadata for {}", path.display()))?;
    let mut permissions = metadata.permissions();
    let mode = permissions.mode();
    let next_mode = mode | 0o001;

    if next_mode != mode {
        permissions.set_mode(next_mode);
        fs::set_permissions(path, permissions)
            .with_context(|| format!("Failed to set mode {:o} on {}", next_mode, path.display()))?;
    }

    Ok(())
}

fn create_release_name() -> Result<String> {
    static TIMESTAMP_FORMAT: &[FormatItem<'static>] = format_description!("[year][month][day]_[hour][minute][second]");
    let now = OffsetDateTime::now_utc();
    now.format(TIMESTAMP_FORMAT).context("Failed to format release timestamp")
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use anyhow::Result;

    use super::ensure_non_owner_can_traverse;

    fn temp_dir_path(test_name: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
        env::temp_dir().join(format!("bonesremote_stage_release_test_{}_{}_{}", process::id(), nanos, test_name))
    }

    /// Adds the other execute bit so non-owner processes can traverse project root parents.
    #[test]
    fn ensure_non_owner_can_traverse_adds_other_execute_bit() -> Result<()> {
        let root = temp_dir_path("other_traverse_bit");
        fs::create_dir_all(&root)?;

        let path = root.join("sites");
        fs::create_dir_all(&path)?;

        let mut permissions = fs::metadata(&path)?.permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&path, permissions)?;

        ensure_non_owner_can_traverse(&path)?;

        let mode = fs::metadata(&path)?.permissions().mode() & 0o777;
        assert_eq!(mode, 0o701);

        fs::remove_dir_all(root)?;
        Ok(())
    }
}
