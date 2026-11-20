use anyhow::Result;
use bonesdeploy_core::config::RemoteDeploymentConfig;

use crate::release::SiteMutation;
use crate::{control_plane, privileges};

pub fn sync(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote config sync")?;
    let descriptor = control_plane::read_stdin_descriptor()?;
    sync_descriptor(site, &descriptor)
}

fn sync_descriptor(site: &str, descriptor: &RemoteDeploymentConfig) -> Result<()> {
    let mutation = SiteMutation::acquire(site)?;
    mutation.ensure_idle()?;
    control_plane::store(site, descriptor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::release::state::{self, DeploymentLock, DeploymentPhase, DeploymentRecord, ProcessIdentity};
    use bonesdeploy_core::config::Bones;
    use std::fs;

    #[test]
    fn config_sync_cannot_replace_a_snapshot_while_the_site_is_locked() -> Result<()> {
        let root = tempfile::tempdir()?;
        let _sites = state::override_sites_root(root.path().join("sites"));
        let _conf = control_plane::override_control_plane_root(root.path().join("conf"));
        let original = RemoteDeploymentConfig::from_bones(&Bones::for_site("atlas"));
        sync_descriptor("atlas", &original)?;
        let _lock = DeploymentLock::acquire("atlas")?;
        let mut replacement = original.clone();
        replacement.releases_keep = 9;

        assert!(sync_descriptor("atlas", &replacement).is_err());
        assert_eq!(control_plane::load("atlas")?, original);
        Ok(())
    }

    #[test]
    fn config_sync_preserves_the_descriptor_of_an_interrupted_activation() -> Result<()> {
        let root = tempfile::tempdir()?;
        let _sites = state::override_sites_root(root.path().join("sites"));
        let _conf = control_plane::override_control_plane_root(root.path().join("conf"));
        let original = RemoteDeploymentConfig::from_bones(&Bones::for_site("atlas"));
        sync_descriptor("atlas", &original)?;
        let record = DeploymentRecord::new(
            "candidate".into(),
            "revision".into(),
            DeploymentPhase::Activated,
            ProcessIdentity::new(0, 0, "interrupted".into()),
        );
        state::write_active_deployment("atlas", &record)?;
        let mut replacement = original.clone();
        replacement.releases_keep = 9;

        assert!(sync_descriptor("atlas", &replacement).is_err());
        assert_eq!(control_plane::load("atlas")?, original);
        Ok(())
    }

    #[test]
    fn deploy_loads_the_descriptor_after_acquiring_the_site_lock() -> Result<()> {
        let root = tempfile::tempdir()?;
        let _sites = state::override_sites_root(root.path().join("sites"));
        let _conf = control_plane::override_control_plane_root(root.path().join("conf"));
        let descriptor = RemoteDeploymentConfig::from_bones(&Bones::for_site("atlas"));
        sync_descriptor("atlas", &descriptor)?;
        let _lock = DeploymentLock::acquire("atlas")?;
        fs::write(control_plane::snapshot_path("atlas"), "invalid json")?;

        let Err(error) = SiteMutation::acquire_for_deploy("atlas") else {
            anyhow::bail!("locked deployment was accepted")
        };
        assert!(error.to_string().contains("already running"));
        Ok(())
    }
}
