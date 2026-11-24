use std::fs;
use std::os::unix::fs::symlink;

use anyhow::Result;
use bonesdeploy_core::{config::Bones, paths};
use bonesremote::release::state::{self, DeploymentLock, DeploymentPhase, DeploymentRecord, ProcessIdentity};
use bonesremote::release::{SiteMutation, cancellation};

#[test]
fn cancellation_removes_only_owned_stale_contexts_and_ignores_symlinks() -> Result<()> {
    let root = tempfile::tempdir()?;
    let _sites = state::override_sites_root(root.path().join("state"));
    let project = root.path().join("project");
    let builds = project.join(paths::TMP_BUILDS_DIR);
    let stale = builds.join("build-demo-1");
    let other = builds.join("build-other-1");
    let outside = root.path().join("outside");
    fs::create_dir_all(&stale)?;
    fs::create_dir_all(&other)?;
    fs::create_dir(&outside)?;
    fs::write(outside.join("keep"), "safe")?;
    let link = builds.join("build-demo-symlink");
    symlink(&outside, &link)?;
    let mut config = Bones::for_site("demo");
    config.project_root = project.display().to_string();
    let mutation = SiteMutation::adopt("demo", config, DeploymentLock::acquire("demo")?)?;

    cancellation::finish_locked(&mutation, "candidate")?;
    assert!(!stale.exists());
    assert!(other.exists());
    assert!(link.is_symlink());
    assert_eq!(fs::read_to_string(outside.join("keep"))?, "safe");
    Ok(())
}

#[test]
fn cancellation_rejects_outside_symlinked_and_noncanonical_recorded_contexts() -> Result<()> {
    for scenario in ["outside", "symlink", "noncanonical"] {
        let root = tempfile::tempdir()?;
        let _sites = state::override_sites_root(root.path().join("state"));
        let project = root.path().join("project");
        let builds = project.join(paths::TMP_BUILDS_DIR);
        let outside = root.path().join("outside");
        fs::create_dir_all(&builds)?;
        fs::create_dir(&outside)?;
        fs::write(outside.join("keep"), "safe")?;
        let recorded = match scenario {
            "symlink" => {
                let link = builds.join("build-demo-1");
                symlink(&outside, &link)?;
                link
            }
            "noncanonical" => {
                fs::create_dir(builds.join("build-demo-1"))?;
                builds.join("build-demo-1").join("..").join("build-demo-1")
            }
            _ => outside.clone(),
        };
        let mut record = DeploymentRecord::new(
            "candidate".into(),
            "revision".into(),
            DeploymentPhase::Received,
            ProcessIdentity::new(0, 0, "interrupted".into()),
        );
        record.set_context(recorded.display().to_string());
        state::write_active_deployment("demo", &record)?;
        let mut config = Bones::for_site("demo");
        config.project_root = project.display().to_string();
        let mutation = SiteMutation::adopt("demo", config, DeploymentLock::acquire("demo")?)?;

        let Err(error) = cancellation::finish_locked(&mutation, "candidate") else {
            anyhow::bail!("invalid recorded context was accepted");
        };
        assert!(error.to_string().contains("invalid build context"), "{error:#}");
        assert_eq!(fs::read_to_string(outside.join("keep"))?, "safe");
        assert!(state::read_active_deployment("demo")?.is_some());
    }
    Ok(())
}
