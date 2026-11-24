use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use anyhow::Result;
use bonesdeploy_core::config::{RemoteDeploymentConfig, RemoteRuntime, RuntimeBackend};
use bonesremote::control_plane::{load, override_control_plane_root, snapshot_path, store};

fn descriptor(backend: RuntimeBackend) -> RemoteDeploymentConfig {
    let runtime = match backend {
        RuntimeBackend::Native => {
            RemoteRuntime::Native { web_root: String::from("public"), ruby_version: None, python_version: None }
        }
        RuntimeBackend::Docker => RemoteRuntime::Docker { compose_port: None, compose_wait_timeout: 120 },
    };
    RemoteDeploymentConfig { releases_keep: 5, runtime }
}

#[test]
fn snapshot_path_uses_the_site_derived_control_plane_location() {
    assert_eq!(snapshot_path("atlas"), PathBuf::from("/srv/conf/atlas/bones.json"));
}

#[test]
fn storing_replaces_snapshot_and_leaves_no_temporary_files() -> Result<()> {
    let root = tempfile::tempdir()?;
    let _scope = override_control_plane_root(root.path().to_path_buf());
    store("atlas", &descriptor(RuntimeBackend::Native))?;
    store("atlas", &descriptor(RuntimeBackend::Docker))?;

    assert_eq!(load("atlas")?.runtime.backend(), RuntimeBackend::Docker);
    assert_eq!(fs::read_dir(root.path().join("atlas"))?.count(), 1);
    Ok(())
}

#[test]
fn stored_snapshot_and_directory_have_restricted_permissions() -> Result<()> {
    let root = tempfile::tempdir()?;
    let _scope = override_control_plane_root(root.path().to_path_buf());
    store("atlas", &descriptor(RuntimeBackend::Native))?;
    store("atlas", &descriptor(RuntimeBackend::Native))?;

    let snapshot = snapshot_path("atlas");
    let directory = snapshot.parent().ok_or_else(|| anyhow::anyhow!("snapshot has no parent"))?;
    assert_eq!(fs::metadata(directory)?.permissions().mode() & 0o777, 0o750);
    assert_eq!(fs::metadata(snapshot)?.permissions().mode() & 0o777, 0o644);
    Ok(())
}

#[test]
fn missing_snapshot_explains_how_to_repair_it() -> Result<()> {
    let root = tempfile::tempdir()?;
    let _scope = override_control_plane_root(root.path().to_path_buf());

    let Err(error) = load("atlas") else {
        anyhow::bail!("snapshot should be missing");
    };
    assert!(error.to_string().contains("bones.json"));
    assert!(error.to_string().contains("bonesdeploy site doctor locally"));
    Ok(())
}

#[test]
fn loading_rejects_unknown_and_unsupported_runtime_fields() -> Result<()> {
    let root = tempfile::tempdir()?;
    let _scope = override_control_plane_root(root.path().to_path_buf());
    let path = snapshot_path("atlas");
    let parent = path.parent().ok_or_else(|| anyhow::anyhow!("snapshot has no parent"))?;
    fs::create_dir_all(parent)?;

    for content in [
        r#"{"releases_keep":5,"runtime":{"backend":"native","web_root":"public"},"bogus":true}"#,
        r#"{"releases_keep":5,"runtime":{"backend":"native","web_root":"public","template":"rails"}}"#,
    ] {
        fs::write(&path, content)?;
        assert!(load("atlas").is_err(), "unexpectedly accepted {content}");
    }
    Ok(())
}

#[test]
fn loading_round_trips_the_narrow_docker_runtime() -> Result<()> {
    let root = tempfile::tempdir()?;
    let _scope = override_control_plane_root(root.path().to_path_buf());
    let expected = RemoteDeploymentConfig {
        releases_keep: 3,
        runtime: RemoteRuntime::Docker { compose_port: Some(8080), compose_wait_timeout: 240 },
    };
    store("atlas", &expected)?;

    assert_eq!(load("atlas")?, expected);
    Ok(())
}
