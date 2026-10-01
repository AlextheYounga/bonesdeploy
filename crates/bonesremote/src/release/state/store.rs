use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use bonesdeploy_core::paths;
use serde::{Deserialize, Serialize};

use super::record::{DeploymentRecord, PreviousDeployment};
use super::{atomic_write, resolved_site_root};

/// The centrally-stored, authoritative per-site deployment state.
///
/// One JSON document per site is the only runtime-mutated state file: the
/// in-flight deployment record, the staged-release pointer, and the reserved
/// future keys. Reads by `list`, `status`, cancellation, recovery, and idle
/// checks and writes by deployment all go through this store, so state is never
/// reconstructed from several files.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SiteState {
    #[serde(default)]
    schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    active: Option<DeploymentRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    staged_release: Option<String>,
}

impl SiteState {
    pub fn active(&self) -> Option<&DeploymentRecord> {
        self.active.as_ref()
    }
    pub fn staged_release(&self) -> Option<&str> {
        self.staged_release.as_deref()
    }
    pub fn with_active(mut self, active: Option<DeploymentRecord>) -> Self {
        self.active = active;
        self
    }
    pub fn with_staged_release(mut self, staged_release: Option<String>) -> Self {
        self.staged_release = staged_release;
        self
    }
}

const SCHEMA_VERSION: u32 = 1;

pub fn state_path(site: &str) -> PathBuf {
    resolved_site_root(site).join(paths::DEPLOYMENT_STATE_FILE)
}

/// Reads the per-site state, migrating previous `active-deployment.json` +
/// `staged-release` files into the centralized store on first read. Returns a
/// fresh (default) state when no store and no previous files exist.
///
/// # Errors
///
/// Returns an error when the store or any previous file exists but cannot be
/// parsed. A malformed file wedges reads (status, cancellation, idle checks)
/// until `release recover` quarantines it.
pub fn read_state(site: &str) -> Result<SiteState> {
    let path = state_path(site);
    if path.is_file() {
        let content = fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))?;
        return serde_json::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()));
    }
    migrate_previous(site)
}

pub fn write_state(site: &str, state: &SiteState) -> Result<()> {
    let content = serde_json::to_string(state).context("Failed to serialize site state")?;
    atomic_write(&state_path(site), content.as_bytes())
        .with_context(|| format!("Failed to write site state at {}", state_path(site).display()))
}

/// Migrates the previous `active-deployment.json` + `staged-release` files into
/// the centralized store, then removes them. No-op (returns a fresh state)
/// when neither file exists.
fn migrate_previous(site: &str) -> Result<SiteState> {
    let site_root = resolved_site_root(site);
    let previous_active_path = site_root.join(paths::ACTIVE_DEPLOYMENT_FILE);
    let previous_staged_path = site_root.join(paths::STAGED_RELEASE_FILE);

    let active = if previous_active_path.is_file() {
        let content = fs::read_to_string(&previous_active_path)
            .with_context(|| format!("Failed to read {}", previous_active_path.display()))?;
        let previous: PreviousDeployment = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse {}", previous_active_path.display()))?;
        Some(previous.to_record()?)
    } else {
        None
    };

    let staged_release = if previous_staged_path.is_file() {
        Some(
            fs::read_to_string(&previous_staged_path)
                .with_context(|| format!("Failed to read {}", previous_staged_path.display()))?
                .trim()
                .to_string(),
        )
    } else {
        None
    };

    if active.is_none() && staged_release.is_none() {
        return Ok(SiteState::default());
    }

    let state = SiteState { schema_version: SCHEMA_VERSION, active, staged_release };
    write_state(site, &state)?;

    if previous_active_path.exists() {
        fs::remove_file(&previous_active_path)
            .with_context(|| format!("Failed to remove migrated previous state {}", previous_active_path.display()))?;
    }
    if previous_staged_path.exists() {
        fs::remove_file(&previous_staged_path)
            .with_context(|| format!("Failed to remove migrated previous state {}", previous_staged_path.display()))?;
    }

    Ok(state)
}

/// Candidate files that may hold malformed state and should be quarantined by
/// recovery when `read_state` fails: the centralized store plus any previous
/// files still on disk.
pub fn quarantine_candidates(site: &str) -> Vec<PathBuf> {
    let site_root = resolved_site_root(site);
    let mut candidates = vec![state_path(site)];
    candidates.push(site_root.join(paths::ACTIVE_DEPLOYMENT_FILE));
    candidates.push(site_root.join(paths::STAGED_RELEASE_FILE));
    candidates
}
