use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use time::OffsetDateTime;
use time::format_description::FormatItem;
use time::macros::format_description;

use crate::config;
use crate::permissions;
use crate::release_state;

pub fn run(config_path: &str) -> Result<()> {
    let cfg = config::load(Path::new(config_path))?;

    let deploy_root = Path::new(&cfg.data.deploy_root);
    let releases_dir = release_state::releases_dir(&cfg);
    let shared_dir = release_state::shared_dir(&cfg);

    fs::create_dir_all(deploy_root)
        .with_context(|| format!("Failed to create deploy_root: {}", deploy_root.display()))?;
    fs::create_dir_all(&releases_dir)
        .with_context(|| format!("Failed to create releases dir: {}", releases_dir.display()))?;
    fs::create_dir_all(&shared_dir)
        .with_context(|| format!("Failed to create shared dir: {}", shared_dir.display()))?;

    let release_name = create_release_name()?;
    let prepared_release_dir = release_state::release_dir(&cfg, &release_name);
    fs::create_dir_all(&prepared_release_dir)
        .with_context(|| format!("Failed to create release dir: {}", prepared_release_dir.display()))?;

    ensure_live_root_symlink(&cfg)?;

    permissions::chown_paths_to_deploy_user(&cfg, &[prepared_release_dir.as_path(), shared_dir.as_path()])?;
    release_state::write_pending_release(&cfg, &release_name)?;

    println!("Prepared release: {release_name}");
    Ok(())
}

fn create_release_name() -> Result<String> {
    static TIMESTAMP_FORMAT: &[FormatItem<'static>] = format_description!("[year][month][day]_[hour][minute][second]");
    let now = OffsetDateTime::now_utc();
    now.format(TIMESTAMP_FORMAT).context("Failed to format release timestamp")
}

fn ensure_live_root_symlink(cfg: &config::BonesConfig) -> Result<()> {
    let live_root = Path::new(&cfg.data.live_root);
    let current_link = release_state::current_link(cfg);

    if live_root.exists() && !live_root.is_symlink() {
        bail!("live_root exists and is not a symlink: {}", live_root.display());
    }

    release_state::point_symlink_atomically(live_root, &current_link)
}
