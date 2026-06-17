use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use time::OffsetDateTime;
use time::format_description::FormatItem;
use time::macros::format_description;

use crate::config;
use crate::release_state;

pub fn run(config_path: &str) -> Result<()> {
    let cfg = config::load(Path::new(config_path))?;

    let project_root = Path::new(&cfg.data.project_root);
    let build_dir = project_root.join(config::Constants::BUILD_DIR);
    let build_root = release_state::build_root(&cfg);
    let releases_dir = release_state::releases_dir(&cfg);
    let shared_dir = release_state::shared_dir(&cfg);

    require_dir(project_root, "project_root")?;
    require_dir(&releases_dir, "releases")?;
    require_dir(&build_dir, "build")?;
    require_dir(&shared_dir, "shared")?;

    fs::create_dir_all(&build_root)
        .with_context(|| format!("Failed to create build workspace: {}", build_root.display()))?;

    let release_name = create_release_name()?;
    let staged_release_dir = release_state::release_dir(&cfg, &release_name);
    fs::create_dir_all(&staged_release_dir)
        .with_context(|| format!("Failed to create release dir: {}", staged_release_dir.display()))?;

    release_state::write_staged_release(&cfg, &release_name)?;

    println!("Staged release: {release_name}");
    Ok(())
}

fn require_dir(path: &Path, label: &str) -> Result<()> {
    if !path.is_dir() {
        bail!(
            "Site not provisioned: {} does not exist ({label}). Run 'bonesdeploy remote setup' first.",
            path.display()
        );
    }
    Ok(())
}

fn create_release_name() -> Result<String> {
    static TIMESTAMP_FORMAT: &[FormatItem<'static>] = format_description!("[year][month][day]_[hour][minute][second]");
    let now = OffsetDateTime::now_utc();
    now.format(TIMESTAMP_FORMAT).context("Failed to format release timestamp")
}
