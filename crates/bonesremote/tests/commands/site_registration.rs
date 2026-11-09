use std::env;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::PathBuf;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;

use bonesdeploy_core::paths;
use bonesremote::release::state::{self, DeploymentLock, ScopedSitesRoot, override_sites_root};

fn temp_root(test_name: &str) -> Result<(ScopedSitesRoot, PathBuf)> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
    let root = env::temp_dir().join(format!("bonesremote_registration_{}_{}_{}", process::id(), nanos, test_name));
    fs::create_dir_all(&root)?;
    Ok((override_sites_root(root.clone()), root))
}

#[test]
fn registration_removal_is_idempotent_and_preserves_the_sibling_lock() -> Result<()> {
    let (_guard, root) = temp_root("remove")?;
    let site_root = root.join("unitapp");
    fs::create_dir_all(site_root.join("nested"))?;
    fs::write(site_root.join("deployment.json"), r#"{"decommissioning":{},"tombstone":{}}"#)?;
    let lock = DeploymentLock::acquire("unitapp")?;

    state::remove_site_registration("unitapp")?;
    state::remove_site_registration("unitapp")?;

    assert!(!site_root.exists());
    assert!(root.join(".unitapp.deployment.lock").is_file());
    drop(lock);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn registration_removal_refuses_symlinks() -> Result<()> {
    let (_guard, root) = temp_root("symlink")?;
    let outside = root.with_extension("outside");
    fs::create_dir_all(&outside)?;
    fs::write(outside.join("keep"), "data")?;
    symlink(&outside, root.join("unitapp"))?;

    assert!(state::remove_site_registration("unitapp").is_err());
    assert!(outside.join("keep").is_file());

    fs::remove_file(root.join("unitapp"))?;
    fs::remove_dir_all(root)?;
    fs::remove_dir_all(outside)?;
    Ok(())
}

#[test]
fn registration_removal_rejects_invalid_site_names() -> Result<()> {
    let (_guard, root) = temp_root("invalid")?;

    assert!(state::remove_site_registration("../outside").is_err());
    assert!(root.is_dir());

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn registration_removal_preserves_backup_key_and_removes_deployment_state() -> Result<()> {
    let (_guard, root) = temp_root("backup")?;
    let site_root = root.join("unitapp");
    fs::create_dir_all(site_root.join("tmp"))?;
    fs::write(site_root.join(paths::DEPLOYMENT_STATE_FILE), "old state")?;
    let passphrase = site_root.join(paths::BORG_PASSPHRASE_FILE);
    fs::write(&passphrase, "retained backup key")?;
    let _lock = DeploymentLock::acquire("unitapp")?;

    state::remove_site_registration("unitapp")?;
    state::remove_site_registration("unitapp")?;

    let retained = root.join(paths::retained_borg_passphrase_filename("unitapp"));
    assert_eq!(fs::read_to_string(retained)?, "retained backup key");
    assert!(!site_root.exists());
    assert!(fs::read_dir(&root)?.all(|entry| entry.is_ok_and(|entry| !entry.path().is_dir())));
    assert!(state::read_site_state("unitapp")?.active().is_none());
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn registration_removal_refuses_a_symlinked_backup_key_before_deleting_state() -> Result<()> {
    let (_guard, root) = temp_root("symlink-key")?;
    let site_root = root.join("unitapp");
    fs::create_dir_all(&site_root)?;
    fs::write(root.join("external-key"), "keep")?;
    symlink(root.join("external-key"), site_root.join(paths::BORG_PASSPHRASE_FILE))?;
    fs::write(site_root.join(paths::DEPLOYMENT_STATE_FILE), "keep state")?;
    let _lock = DeploymentLock::acquire("unitapp")?;

    assert!(state::remove_site_registration("unitapp").is_err());
    assert!(site_root.join(paths::DEPLOYMENT_STATE_FILE).is_file());
    assert_eq!(fs::read_to_string(root.join("external-key"))?, "keep");
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn registration_removal_cannot_overwrite_a_different_retained_backup_key() -> Result<()> {
    let (_guard, root) = temp_root("older-key")?;
    let site_root = root.join("unitapp");
    fs::create_dir_all(&site_root)?;
    fs::write(site_root.join(paths::BORG_PASSPHRASE_FILE), "current key")?;
    let retained = root.join(paths::retained_borg_passphrase_filename("unitapp"));
    fs::write(&retained, "older key")?;
    let _lock = DeploymentLock::acquire("unitapp")?;

    assert!(state::remove_site_registration("unitapp").is_err());
    assert_eq!(fs::read_to_string(retained)?, "older key");
    assert!(site_root.is_dir());
    fs::remove_dir_all(root)?;
    Ok(())
}
