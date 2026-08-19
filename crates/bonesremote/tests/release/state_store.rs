use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use bonesdeploy_core::paths;

use bonesremote::release::state::record::{DeploymentPhase, DeploymentRecord};
use bonesremote::release::state::store::{SiteState, read_state, state_path, write_state};
use bonesremote::release::state::{ProcessIdentity, ScopedSitesRoot, override_sites_root};

fn temp_root(test_name: &str) -> Result<(ScopedSitesRoot, PathBuf)> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
    let path = env::temp_dir().join(format!("bonesremote_store_{}_{}_{}", process::id(), nanos, test_name));
    fs::create_dir_all(&path)?;
    Ok((override_sites_root(path.clone()), path))
}

#[test]
fn missing_state_reads_as_default_without_writing() -> Result<()> {
    let (_guard, root) = temp_root("missing")?;

    let state = read_state("unitapp")?;

    assert!(state.active().is_none());
    assert!(state.staged_release().is_none());
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
        ProcessIdentity::new(1234, 42, String::from("2026-08-04T19:03:21Z")),
    );
    let state = SiteState::default().with_active(Some(record)).with_staged_release(Some(String::from("staged")));

    write_state("unitapp", &state)?;
    let loaded = read_state("unitapp")?;

    let active = loaded.active().ok_or_else(|| anyhow::anyhow!("active record must round-trip"))?;
    assert_eq!(active.release(), "20260804_190321-46a0b75c-a7f2");
    assert_eq!(loaded.staged_release(), Some("staged"));
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
        site_root.join(paths::ACTIVE_DEPLOYMENT_FILE),
        r#"{"release":"20260101_000000","pid":1234,"phase":"building","started_at":"2026-08-04T19:03:21Z"}"#,
    )?;
    fs::write(site_root.join(paths::STAGED_RELEASE_FILE), "20260101_000000\n")?;

    let state = read_state("unitapp")?;

    let active = state.active().ok_or_else(|| anyhow::anyhow!("previous active deployment must be migrated"))?;
    assert_eq!(active.phase(), &DeploymentPhase::Created);
    assert_eq!(state.staged_release(), Some("20260101_000000"));
    assert!(state_path("unitapp").exists(), "migration must persist the store");
    assert!(!site_root.join(paths::ACTIVE_DEPLOYMENT_FILE).exists(), "previous active file must be removed");
    assert!(!site_root.join(paths::STAGED_RELEASE_FILE).exists(), "previous staged file must be removed");
    fs::remove_dir_all(root).ok();
    Ok(())
}
