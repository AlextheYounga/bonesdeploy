use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use shared::paths;
use shared::registry;
use time::OffsetDateTime;
use time::format_description::FormatItem;
use time::macros::format_description;

use crate::privileges;
use crate::release_state;

pub fn run(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote release stage")?;

    let registry_path = paths::bonesremote_registry_path(site);
    let cfg = registry::load(&registry_path)
        .with_context(|| format!("Failed to load remote site state from {}", registry_path.display()))?;

    let project_root = Path::new(&cfg.site_root);
    require_dir(project_root, "project_root directory")?;
    require_dir(&release_state::releases_dir(&cfg), "releases")?;
    require_dir(&release_state::shared_dir(&cfg), "shared")?;

    let release_name = create_release_name()?;
    let release_dir = release_state::release_dir(&cfg, &release_name);
    fs::create_dir_all(&release_dir)
        .with_context(|| format!("Failed to create release dir: {}", release_dir.display()))?;

    release_state::write_staged_release(site, &release_name)?;

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

#[cfg(test)]
mod tests {}
