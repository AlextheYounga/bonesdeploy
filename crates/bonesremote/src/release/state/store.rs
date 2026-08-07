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
    pub schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active: Option<DeploymentRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub staged_release: Option<String>,
}

const SCHEMA_VERSION: u32 = 1;

pub(crate) fn state_path(site: &str) -> PathBuf {
    resolved_site_root(site).join(paths::deployment_state_file())
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
pub(crate) fn read_state(site: &str) -> Result<SiteState> {
    let path = state_path(site);
    if path.is_file() {
        let content = fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))?;
        return serde_json::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()));
    }
    migrate_previous(site)
}

pub(crate) fn write_state(site: &str, state: &SiteState) -> Result<()> {
    let content = serde_json::to_string(state).context("Failed to serialize site state")?;
    atomic_write(&state_path(site), content.as_bytes())
        .with_context(|| format!("Failed to write site state at {}", state_path(site).display()))
}

/// Migrates the previous `active-deployment.json` + `staged-release` files into
/// the centralized store, then removes them. No-op (returns a fresh state)
/// when neither file exists.
fn migrate_previous(site: &str) -> Result<SiteState> {
    let site_root = resolved_site_root(site);
    let previous_active_path = site_root.join(paths::active_deployment_file());
    let previous_staged_path = site_root.join(paths::staged_release_file());

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
pub(crate) fn quarantine_candidates(site: &str) -> Vec<PathBuf> {
    let site_root = resolved_site_root(site);
    let mut candidates = vec![state_path(site)];
    candidates.push(site_root.join(paths::active_deployment_file()));
    candidates.push(site_root.join(paths::staged_release_file()));
    candidates
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use anyhow::Result;
    use bonesdeploy_core::paths;

    use super::{SiteState, read_state, state_path, write_state};
    use crate::release::state::record::{DeploymentPhase, DeploymentRecord};
    use crate::release::state::{ScopedRoot, set_sites_root_for_tests};

    fn temp_root(test_name: &str) -> Result<(ScopedRoot, PathBuf)> {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
        let path = env::temp_dir().join(format!("bonesremote_store_{}_{}_{}", process::id(), nanos, test_name));
        fs::create_dir_all(&path)?;
        Ok((set_sites_root_for_tests(path.clone()), path))
    }

    #[test]
    fn missing_state_reads_as_default_without_writing() -> Result<()> {
        let (_guard, root) = temp_root("missing")?;

        let state = read_state("unitapp")?;

        assert!(state.active.is_none());
        assert!(state.staged_release.is_none());
        assert!(!state_path("unitapp").exists());
        fs::remove_dir_all(root).ok();
        Ok(())
    }

    #[test]
    fn state_round_trips_through_json() -> Result<()> {
        let (_guard, root) = temp_root("round_trip")?;
        let record = DeploymentRecord::new(
            String::from("20260804_190321-46a0b75c-a7f2"),
            String::from("46a0b75c"),
            DeploymentPhase::Verified,
            1234,
            42,
            String::from("2026-08-04T19:03:21Z"),
        );
        let state = SiteState { schema_version: 1, active: Some(record), staged_release: Some(String::from("staged")) };

        write_state("unitapp", &state)?;
        let loaded = read_state("unitapp")?;

        let active = loaded.active.ok_or_else(|| anyhow::anyhow!("active record must round-trip"))?;
        assert_eq!(active.release, "20260804_190321-46a0b75c-a7f2");
        assert_eq!(loaded.staged_release.as_deref(), Some("staged"));
        fs::remove_dir_all(root).ok();
        Ok(())
    }

    #[test]
    fn malformed_store_fails_to_parse() -> Result<()> {
        let (_guard, root) = temp_root("malformed")?;
        let state_path = state_path("unitapp");
        if let Some(parent) = state_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&state_path, "{ not valid json")?;

        assert!(read_state("unitapp").is_err());
        fs::remove_dir_all(root).ok();
        Ok(())
    }

    #[test]
    fn previous_active_and_staged_are_migrated_into_the_store() -> Result<()> {
        let (_guard, root) = temp_root("migrate")?;
        let site_root = root.join("unitapp");
        fs::create_dir_all(&site_root)?;
        fs::write(
            site_root.join(paths::active_deployment_file()),
            r#"{"release":"20260101_000000","pid":1234,"phase":"building","started_at":"2026-08-04T19:03:21Z"}"#,
        )?;
        fs::write(site_root.join(paths::staged_release_file()), "20260101_000000\n")?;

        let state = read_state("unitapp")?;

        let active = state.active.ok_or_else(|| anyhow::anyhow!("previous active deployment must be migrated"))?;
        assert_eq!(active.phase, DeploymentPhase::Created);
        assert_eq!(state.staged_release.as_deref(), Some("20260101_000000"));
        assert!(state_path("unitapp").exists(), "migration must persist the store");
        assert!(!site_root.join(paths::active_deployment_file()).exists(), "previous active file must be removed");
        assert!(!site_root.join(paths::staged_release_file()).exists(), "previous staged file must be removed");
        fs::remove_dir_all(root).ok();
        Ok(())
    }
}
