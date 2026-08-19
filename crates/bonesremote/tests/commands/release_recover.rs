use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use bonesdeploy_core::paths;

use bonesremote::commands::release::recover::recover_site;
use bonesremote::release::state::{
    self as release_state, DeploymentPhase, DeploymentRecord, ScopedSitesRoot, override_sites_root,
};

fn temp_root(test_name: &str) -> Result<(ScopedSitesRoot, PathBuf)> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
    let path = env::temp_dir().join(format!("bonesremote_recover_{test_name}_{}_{}", process::id(), nanos));
    fs::create_dir_all(&path)?;
    Ok((override_sites_root(path.clone()), path))
}

#[test]
fn recover_quarantines_malformed_store_state() -> Result<()> {
    let (_guard, root) = temp_root("malformed")?;
    let site_root = root.join("unitapp");
    fs::create_dir_all(&site_root)?;
    fs::write(site_root.join(paths::DEPLOYMENT_STATE_FILE), "{ not valid json")?;

    recover_site("unitapp")?;

    let quarantine = site_root.join(paths::RECOVERY_DIR);
    assert!(fs::read_dir(&quarantine)?.next().is_some(), "malformed state must be quarantined");
    assert!(!site_root.join(paths::DEPLOYMENT_STATE_FILE).exists());

    fs::remove_dir_all(root).ok();
    Ok(())
}

#[test]
fn recover_is_noop_when_state_is_valid() -> Result<()> {
    let (_guard, root) = temp_root("valid")?;
    let record = DeploymentRecord::new(
        String::from("20260804_190321-46a0b75c-0000"),
        String::from("46a0b75c"),
        DeploymentPhase::Activated,
        release_state::ProcessIdentity::new(process::id(), 0, String::from("2026-08-04T19:03:21Z")),
    );
    release_state::write_active_deployment("unitapp", &record)?;

    recover_site("unitapp")?;

    assert!(release_state::read_active_deployment("unitapp")?.is_some());
    assert!(!root.join("unitapp").join(paths::RECOVERY_DIR).exists());

    fs::remove_dir_all(root).ok();
    Ok(())
}

#[test]
fn recover_migrates_previous_state_instead_of_quarantining() -> Result<()> {
    let (_guard, root) = temp_root("previous")?;
    let site_root = root.join("unitapp");
    fs::create_dir_all(&site_root)?;
    fs::write(
        site_root.join(paths::ACTIVE_DEPLOYMENT_FILE),
        r#"{"release":"20260101_000000","pid":1234,"phase":"preparing","started_at":"2026-08-04T19:03:21Z"}"#,
    )?;

    recover_site("unitapp")?;

    assert!(site_root.join(paths::DEPLOYMENT_STATE_FILE).exists(), "previous state must migrate into the store");
    assert!(!site_root.join(paths::ACTIVE_DEPLOYMENT_FILE).exists());
    let active = release_state::read_active_deployment("unitapp")?
        .ok_or_else(|| anyhow::anyhow!("migrated active deployment must be present"))?;
    assert_eq!(active.phase(), &DeploymentPhase::Prepared);
    fs::remove_dir_all(root).ok();
    Ok(())
}

#[test]
fn recover_refuses_while_a_deployment_holds_the_lock() -> Result<()> {
    let (_guard, _root) = temp_root("locked")?;
    let _lock = release_state::DeploymentLock::acquire("unitapp")?;

    assert!(recover_site("unitapp").is_err());
    Ok(())
}
