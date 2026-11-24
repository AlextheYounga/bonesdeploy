use anyhow::Result;
use bonesdeploy_core::config::{Bones, RemoteDeploymentConfig};
use bonesremote::control_plane;
use bonesremote::release::state::{self, DeploymentLock, DeploymentPhase, DeploymentRecord, ProcessIdentity};

#[test]
fn sync_cannot_replace_a_snapshot_while_the_site_is_locked() -> Result<()> {
    let root = tempfile::tempdir()?;
    let _sites = state::override_sites_root(root.path().join("sites"));
    let _conf = control_plane::override_control_plane_root(root.path().join("conf"));
    let original = RemoteDeploymentConfig::from_bones(&Bones::for_site("atlas"));
    control_plane::sync("atlas", &original)?;
    let lock = DeploymentLock::acquire("atlas")?;
    let mut replacement = original.clone();
    replacement.releases_keep = 9;

    assert!(control_plane::sync("atlas", &replacement).is_err());
    assert_eq!(control_plane::load("atlas")?, original);
    drop(lock);
    control_plane::sync("atlas", &replacement)?;
    assert_eq!(control_plane::load("atlas")?, replacement);
    Ok(())
}

#[test]
fn sync_preserves_the_snapshot_of_an_interrupted_activation() -> Result<()> {
    let root = tempfile::tempdir()?;
    let _sites = state::override_sites_root(root.path().join("sites"));
    let _conf = control_plane::override_control_plane_root(root.path().join("conf"));
    let original = RemoteDeploymentConfig::from_bones(&Bones::for_site("atlas"));
    control_plane::sync("atlas", &original)?;
    let record = DeploymentRecord::new(
        "candidate".into(),
        "revision".into(),
        DeploymentPhase::Activated,
        ProcessIdentity::new(0, 0, "interrupted".into()),
    );
    state::write_active_deployment("atlas", &record)?;
    let mut replacement = original.clone();
    replacement.releases_keep = 9;

    assert!(control_plane::sync("atlas", &replacement).is_err());
    assert_eq!(control_plane::load("atlas")?, original);
    Ok(())
}
