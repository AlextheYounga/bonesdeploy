use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};

use crate::privileges;
use crate::release::state::{self as release_state, DeploymentLock};

pub fn run(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote release recover")?;
    recover_site(site)
}

/// This is intentionally the one command that reads state without a
/// `SiteMutation`: malformed state cannot be loaded into one. It quarantines
/// malformed centralized deployment state so the site becomes
/// recoverable (status, cancellation, and idle checks all parse the store).
///
/// A running deployment holds the site's deployment lock for its entire
/// lifetime, so acquiring the lock here proves no deployment process is alive:
/// only then is it safe to move malformed state aside without racing the
/// deployer that wrote it.
pub fn recover_site(site: &str) -> Result<()> {
    let _lock = DeploymentLock::acquire(site)
        .with_context(|| format!("Refusing to recover {site} while a deployment is running"))?;

    match release_state::read_site_state(site) {
        Ok(state) => {
            if state.active().is_none() && state.staged_release().is_none() {
                println!("No deployment state for {site}.");
            } else {
                println!("Deployment state for {site} is valid; nothing to recover.");
            }
        }
        Err(parse_error) => {
            quarantine_malformed(site)?;
            println!("Quarantined malformed deployment state for {site}\n  not parseable: {parse_error}");
        }
    }
    Ok(())
}

fn quarantine_malformed(site: &str) -> Result<()> {
    let quarantine_dir = release_state::recovery_dir(site);
    fs::create_dir_all(&quarantine_dir)
        .with_context(|| format!("Failed to create quarantine directory {}", quarantine_dir.display()))?;
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());

    for source in release_state::quarantine_candidates(site) {
        if !source.is_file() {
            continue;
        }
        let file_name = source.file_name().context("Quarantine source has no file name")?;
        let target = quarantine_dir.join(format!("{}-{stamp}.json", file_name.to_string_lossy()));
        fs::rename(&source, &target)
            .with_context(|| format!("Failed to quarantine {} into {}", source.display(), target.display()))?;
        println!("  moved {}", source.display());
    }
    Ok(())
}
