use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use serde_json::json;

use bonesremote::release::SiteMutation;
use bonesremote::release::state::{self, DeletionPlan, DeploymentLock, ScopedSitesRoot, override_sites_root};

fn temp_root(test_name: &str) -> Result<(ScopedSitesRoot, PathBuf)> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
    let root = env::temp_dir().join(format!("bonesremote_decommission_{}_{}_{}", process::id(), nanos, test_name));
    fs::create_dir_all(&root)?;
    Ok((override_sites_root(root.clone()), root))
}

#[test]
fn begin_reuses_the_first_plan_and_blocks_normal_mutations() -> Result<()> {
    let (_guard, root) = temp_root("begin")?;
    let first = DeletionPlan::new(json!({"resources": ["first"]}));
    let second = DeletionPlan::new(json!({"resources": ["second"]}));

    let lock = DeploymentLock::acquire("unitapp")?;
    assert_eq!(state::begin_decommission("unitapp", first)?.value(), &json!({"resources": ["first"]}));
    drop(lock);

    let lock = DeploymentLock::acquire("unitapp")?;
    assert_eq!(state::begin_decommission("unitapp", second)?.value(), &json!({"resources": ["first"]}));
    drop(lock);

    assert!(SiteMutation::adopt("unitapp", Default::default(), DeploymentLock::acquire("unitapp")?).is_err());
    fs::remove_dir_all(root).ok();
    Ok(())
}

#[test]
fn completion_retains_the_plan_in_an_unverified_tombstone_until_verified() -> Result<()> {
    let (_guard, root) = temp_root("complete")?;
    let plan = DeletionPlan::new(json!({"version": 1}));

    let lock = DeploymentLock::acquire("unitapp")?;
    state::begin_decommission("unitapp", plan)?;
    state::complete_decommission("unitapp")?;
    drop(lock);

    let state_after_complete = state::read_site_state("unitapp")?;
    let tombstone = state_after_complete.tombstone().ok_or_else(|| anyhow::anyhow!("tombstone must exist"))?;
    assert!(!tombstone.verified());
    assert_eq!(tombstone.plan().value(), &json!({"version": 1}));
    assert!(SiteMutation::adopt("unitapp", Default::default(), DeploymentLock::acquire("unitapp")?).is_err());

    let lock = DeploymentLock::acquire("unitapp")?;
    state::verify_decommission("unitapp")?;
    drop(lock);
    assert!(state::read_site_state("unitapp")?.tombstone().is_some_and(|tombstone| tombstone.verified()));

    fs::remove_dir_all(root).ok();
    Ok(())
}

#[test]
fn reactivation_requires_verification_and_reopens_mutations_afterward() -> Result<()> {
    let (_guard, root) = temp_root("reactivate")?;
    let plan = DeletionPlan::new(json!({"version": 1}));

    let lock = DeploymentLock::acquire("unitapp")?;
    state::begin_decommission("unitapp", plan)?;
    state::complete_decommission("unitapp")?;
    assert!(state::reactivate_decommission("unitapp").is_err());
    state::verify_decommission("unitapp")?;
    state::reactivate_decommission("unitapp")?;
    drop(lock);

    assert!(state::read_site_state("unitapp")?.tombstone().is_none());
    let lock = DeploymentLock::acquire("unitapp")?;
    assert!(SiteMutation::adopt("unitapp", Default::default(), lock).is_ok());

    fs::remove_dir_all(root).ok();
    Ok(())
}
