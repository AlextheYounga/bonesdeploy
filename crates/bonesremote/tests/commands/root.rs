use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;

use bonesremote::commands::ensure_site_idle;
use bonesremote::release::SiteMutation;
use bonesremote::release::state::{self as release_state, DeploymentPhase, DeploymentRecord, ProcessIdentity};

fn temp_root(test_name: &str) -> Result<PathBuf> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
    let path = env::temp_dir().join(format!("bonesremote_idle_{}_{}_{}", process::id(), nanos, test_name));
    fs::create_dir_all(&path)?;
    Ok(path)
}

fn record(phase: DeploymentPhase) -> DeploymentRecord {
    DeploymentRecord::new(
        String::from("20260804_190321-46a0b75c-a7f2"),
        String::from("46a0b75c"),
        phase,
        ProcessIdentity::new(process::id(), 0, String::from("2026-08-04T19:03:21Z")),
    )
}

#[test]
fn pre_commit_active_deployment_blocks_site_mutation() -> Result<()> {
    let root = temp_root("pre_commit")?;
    let _guard = release_state::override_sites_root(root.clone());
    let mutation =
        SiteMutation::adopt("unitapp", Default::default(), release_state::DeploymentLock::acquire("unitapp")?);

    release_state::write_active_deployment("unitapp", &record(DeploymentPhase::Prepared))?;

    assert!(ensure_site_idle(&mutation).is_err());
    fs::remove_dir_all(root).ok();
    Ok(())
}

#[test]
fn committed_deployment_is_serialization_idle() -> Result<()> {
    let root = temp_root("committed")?;
    let _guard = release_state::override_sites_root(root.clone());
    let mutation =
        SiteMutation::adopt("unitapp", Default::default(), release_state::DeploymentLock::acquire("unitapp")?);

    release_state::write_active_deployment("unitapp", &record(DeploymentPhase::CleanupPending))?;

    assert!(ensure_site_idle(&mutation).is_ok(), "cleanup_pending must never block the next deployment");
    fs::remove_dir_all(root).ok();
    Ok(())
}

#[test]
fn staged_release_without_committed_deployment_blocks_site_mutation() -> Result<()> {
    let root = temp_root("staged")?;
    let _guard = release_state::override_sites_root(root.clone());
    let mutation =
        SiteMutation::adopt("unitapp", Default::default(), release_state::DeploymentLock::acquire("unitapp")?);

    release_state::write_staged_release("unitapp", "20260804_190321-46a0b75c-a7f2")?;

    assert!(ensure_site_idle(&mutation).is_err());
    fs::remove_dir_all(root).ok();
    Ok(())
}
