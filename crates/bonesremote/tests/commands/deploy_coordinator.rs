use std::cell::RefCell;
use std::fs;
use std::os::unix::fs::symlink;

use anyhow::{Result, bail};
use bonesremote::release::lifecycle::activate::{ActivationFailure, ReleaseActivation};
use bonesremote::release::state::{self, DeploymentPhase, DeploymentRecord, ProcessIdentity};

#[test]
fn persistence_and_restart_failures_leave_the_previous_release_verified() -> Result<()> {
    for failure in ["sealed-write", "activated-write", "restart", "verified-write"] {
        let root = tempfile::tempdir()?;
        let previous = root.path().join("previous");
        let candidate = root.path().join("candidate");
        fs::create_dir(&previous)?;
        fs::create_dir(&candidate)?;
        let current = root.path().join("current");
        symlink(&previous, &current)?;
        let restarted = RefCell::new(Vec::new());

        let result = ReleaseActivation::new(root.path(), &candidate, &previous).run(
            |phase| {
                let expected_current = if phase == DeploymentPhase::Sealed { &previous } else { &candidate };
                assert_eq!(fs::read_link(&current)?, *expected_current);
                if matches!(
                    (&phase, failure),
                    (DeploymentPhase::Sealed, "sealed-write")
                        | (DeploymentPhase::Activated, "activated-write")
                        | (DeploymentPhase::Verified, "verified-write")
                ) {
                    bail!("{failure} failed")
                }
                Ok(())
            },
            || {
                let target = fs::read_link(&current)?;
                restarted.borrow_mut().push(target.clone());
                if target == candidate && failure == "restart" {
                    bail!("candidate restart failed")
                }
                Ok(())
            },
        );

        assert!(matches!(result, Err(ActivationFailure::Aborted(_))), "{failure}");
        assert_eq!(fs::read_link(&current)?, previous, "{failure}");
        let expected = if failure == "sealed-write" {
            vec![]
        } else if failure == "activated-write" {
            vec![previous.clone()]
        } else {
            vec![candidate.clone(), previous.clone()]
        };
        assert_eq!(*restarted.borrow(), expected, "{failure}");
        assert!(candidate.is_dir(), "the cutover boundary must not delete recovery evidence");
    }
    Ok(())
}

#[test]
fn unsuccessful_restoration_requires_recovery_instead_of_candidate_cleanup() -> Result<()> {
    for missing_previous in [true, false] {
        let root = tempfile::tempdir()?;
        let previous = root.path().join("previous");
        let candidate = root.path().join("candidate");
        if !missing_previous {
            fs::create_dir(&previous)?;
        }
        fs::create_dir(&candidate)?;
        let current = root.path().join("current");
        symlink(&previous, &current)?;

        let result = ReleaseActivation::new(root.path(), &candidate, &previous).run(
            |phase| {
                if phase == DeploymentPhase::Activated {
                    bail!("state write failed")
                }
                Ok(())
            },
            || bail!("restored service failed"),
        );

        let Err(ActivationFailure::RecoveryRequired(error)) = result else {
            bail!("failed restoration must retain recovery state")
        };
        assert!(format!("{error:#}").contains("state write failed"));
        assert!(candidate.is_dir());
        assert_eq!(fs::read_link(&current)?, if missing_previous { candidate } else { previous });
    }
    Ok(())
}

#[test]
fn activation_persists_its_restoration_target_before_switching_and_commits_after_verification() -> Result<()> {
    let root = tempfile::tempdir()?;
    let previous = root.path().join("previous");
    let candidate = root.path().join("candidate");
    fs::create_dir(&previous)?;
    fs::create_dir(&candidate)?;
    let current = root.path().join("current");
    symlink(&previous, &current)?;
    let events = RefCell::new(Vec::new());
    let state_path = root.path().join("deployment.json");
    let mut record = DeploymentRecord::new(
        "candidate".into(),
        "revision".into(),
        DeploymentPhase::Created,
        ProcessIdentity::new(1, 1, "now".into()),
    );
    record.set_previous_release(Some("previous".into()));

    let result = ReleaseActivation::new(root.path(), &candidate, &previous).run(
        |phase| {
            let event = match phase {
                DeploymentPhase::Sealed => {
                    assert_eq!(fs::read_link(&current)?, previous);
                    "sealed"
                }
                DeploymentPhase::Activated => "activated",
                DeploymentPhase::Verified => "verified",
                _ => bail!("unexpected activation phase"),
            };
            events.borrow_mut().push(event);
            record.set_phase(phase);
            state::atomic_write(&state_path, &serde_json::to_vec(&record)?)
        },
        || {
            let persisted: serde_json::Value = serde_json::from_slice(&fs::read(&state_path)?)?;
            assert_eq!(persisted["previous_release"], "previous");
            assert_eq!(persisted["phase"], "activated");
            events.borrow_mut().push("restart");
            Ok(())
        },
    );
    assert!(result.is_ok());
    assert_eq!(*events.borrow(), ["sealed", "activated", "restart", "verified"]);
    assert_eq!(fs::read_link(&current)?, candidate);
    let persisted: DeploymentRecord = serde_json::from_slice(&fs::read(state_path)?)?;
    assert_eq!(persisted.phase(), &DeploymentPhase::Verified);
    Ok(())
}

#[test]
fn invalid_candidate_leaves_the_original_release_untouched() -> Result<()> {
    let root = tempfile::tempdir()?;
    let previous = root.path().join("previous");
    fs::create_dir(&previous)?;
    let current = root.path().join("current");
    symlink(&previous, &current)?;
    let result = ReleaseActivation::new(root.path(), &root.path().join("missing"), &previous).run(
        |phase| {
            assert_eq!(phase, DeploymentPhase::Sealed);
            Ok(())
        },
        || bail!("must not restart after a failed switch"),
    );
    assert!(matches!(result, Err(ActivationFailure::Aborted(_))));
    assert_eq!(fs::read_link(current)?, previous);
    Ok(())
}
