use std::env;
use std::fs;
use std::os::unix::fs::symlink;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use bonesdeploy_core::config::Bones;
use bonesdeploy_core::paths;

use bonesremote::commands::drop_failed_release::ensure_release_not_active;
use bonesremote::release::SiteMutation;
use bonesremote::release::state::{DeploymentLock, override_sites_root};

#[test]
fn active_release_cannot_be_dropped() -> Result<()> {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let root = env::temp_dir().join(format!("bonesremote_drop_{}_{}", process::id(), nonce));
    let _root_guard = override_sites_root(root.clone());
    let release = root.join(paths::RELEASES_DIR).join("active-release");
    fs::create_dir_all(&release)?;
    symlink(&release, root.join(paths::CURRENT_LINK))?;

    let mut config = Bones::for_site("demo");
    config.project_root = root.to_string_lossy().into_owned();
    let mutation = SiteMutation::adopt("demo", config, DeploymentLock::acquire("demo")?);
    assert!(ensure_release_not_active(&mutation, "active-release").is_err());

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn cleanup_requires_a_readable_active_release() -> Result<()> {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let root = env::temp_dir().join(format!("bonesremote_drop_missing_current_{}_{}", process::id(), nonce));
    let _root_guard = override_sites_root(root.clone());
    fs::create_dir_all(&root)?;

    let mut config = Bones::for_site("demo");
    config.project_root = root.to_string_lossy().into_owned();
    let mutation = SiteMutation::adopt("demo", config, DeploymentLock::acquire("demo")?);
    assert!(ensure_release_not_active(&mutation, "candidate").is_err());

    fs::remove_dir_all(root)?;
    Ok(())
}
