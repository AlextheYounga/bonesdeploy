use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use bonesdeploy_core::config::Bones;
use bonesremote::commands::backup;
use bonesremote::release::SiteMutation;
use bonesremote::release::state::{DeploymentLock, override_sites_root};

#[test]
fn backup_rejects_a_site_without_a_provisioned_passphrase() -> Result<()> {
    let root = tempfile::tempdir()?;
    let _sites = override_sites_root(root.path().join("sites"));
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let site = format!("backup-test-{}-{nonce}", process::id());
    let mutation = SiteMutation::adopt(&site, Bones::for_site(&site), DeploymentLock::acquire(&site)?)?;

    let Err(error) = backup::run_locked(&mutation, 14) else {
        anyhow::bail!("backup without a provisioned passphrase unexpectedly succeeded");
    };
    assert!(error.to_string().contains("Missing Borg passphrase"), "{error:#}");
    assert!(error.to_string().contains("is the site provisioned for backups?"), "{error:#}");
    Ok(())
}
