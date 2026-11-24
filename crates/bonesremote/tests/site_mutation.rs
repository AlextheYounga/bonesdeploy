use anyhow::Result;
use bonesdeploy_core::config::Bones;
use bonesremote::release::SiteMutation;
use bonesremote::release::state::{self, DeploymentLock, DeploymentPhase, DeploymentRecord, ProcessIdentity};

#[test]
fn interrupted_activation_prevents_an_idle_site_mutation() -> Result<()> {
    let root = tempfile::tempdir()?;
    let _sites = state::override_sites_root(root.path().join("sites"));
    let record = DeploymentRecord::new(
        "candidate".into(),
        "revision".into(),
        DeploymentPhase::Activated,
        ProcessIdentity::new(0, 0, "interrupted".into()),
    );
    state::write_active_deployment("atlas", &record)?;
    let lock = DeploymentLock::acquire("atlas")?;
    let mutation = SiteMutation::adopt("atlas", Bones::for_site("atlas"), lock)?;

    let Err(error) = mutation.ensure_idle() else {
        anyhow::bail!("interrupted activation must prevent mutation");
    };
    assert!(error.to_string().contains("interrupted"));
    assert_eq!(state::read_active_deployment("atlas")?.as_ref().map(DeploymentRecord::release), Some("candidate"));
    Ok(())
}
