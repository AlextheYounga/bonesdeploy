use std::fs;
use std::io::Cursor;

use anyhow::Result;
use bonesdeploy_core::artifact::{ArtifactManifest, ComposeImage, write_manifest};
use bonesdeploy_core::config::{RemoteDeploymentConfig, RemoteRuntime};
use bonesremote::control_plane;
use bonesremote::release::lifecycle::request;
use bonesremote::release::state::{DeploymentLock, override_sites_root};

const REVISION: &str = "0123456789abcdef0123456789abcdef01234567";

fn descriptor(docker: bool) -> RemoteDeploymentConfig {
    let runtime = if docker {
        RemoteRuntime::Docker { compose_port: None, compose_wait_timeout: 120 }
    } else {
        RemoteRuntime::Native { web_root: "public".into(), ruby_version: None, python_version: None }
    };
    RemoteDeploymentConfig { releases_keep: 5, runtime }
}

fn frame(manifest: &ArtifactManifest) -> Result<Cursor<Vec<u8>>> {
    let mut bytes = Vec::new();
    write_manifest(&mut bytes, manifest)?;
    Ok(Cursor::new(bytes))
}

#[test]
fn deployment_acquires_the_lock_before_reading_the_descriptor_or_artifact() -> Result<()> {
    let root = tempfile::tempdir()?;
    let _sites = override_sites_root(root.path().join("sites"));
    let _conf = control_plane::override_control_plane_root(root.path().join("conf"));
    control_plane::store("atlas", &descriptor(false))?;
    let lock = DeploymentLock::acquire("atlas")?;
    fs::write(control_plane::snapshot_path("atlas"), "invalid json")?;
    let mut input = Cursor::new(b"not a manifest".to_vec());

    let Err(error) = request::receive("atlas", &mut input) else {
        anyhow::bail!("locked deployment was accepted");
    };
    assert!(error.to_string().contains("already running"), "{error:#}");
    assert_eq!(input.position(), 0);
    drop(lock);
    let Err(error) = request::receive("atlas", &mut input) else {
        anyhow::bail!("invalid descriptor was accepted");
    };
    assert!(format!("{error:#}").contains("parse"), "{error:#}");
    assert_eq!(input.position(), 0);
    Ok(())
}

#[test]
fn deployment_accepts_only_artifacts_matching_the_configured_backend_and_site() -> Result<()> {
    let root = tempfile::tempdir()?;
    let _sites = override_sites_root(root.path().join("sites"));
    let _conf = control_plane::override_control_plane_root(root.path().join("conf"));
    let native = ArtifactManifest::new_native_tree("atlas".into(), REVISION.into(), 1, &"a".repeat(64));
    let compose = native.clone().with_compose_images(vec![ComposeImage::new("atlas", "web".into(), REVISION)?]);
    for (docker, manifest, accepted) in
        [(false, &native, true), (false, &compose, false), (true, &native, false), (true, &compose, true)]
    {
        control_plane::store("atlas", &descriptor(docker))?;
        let result = request::receive("atlas", &mut frame(manifest)?);
        assert_eq!(result.is_ok(), accepted, "docker={docker}, manifest={:?}", manifest.kind);
        if let Ok((guard, _)) = result {
            assert!(DeploymentLock::acquire("atlas").is_err(), "successful request must retain the site lock");
            drop(guard);
        }
    }
    let other = ArtifactManifest::new_native_tree("other".into(), REVISION.into(), 1, &"a".repeat(64));
    control_plane::store("atlas", &descriptor(false))?;
    let Err(error) = request::receive("atlas", &mut frame(&other)?) else {
        anyhow::bail!("artifact for another site was accepted");
    };
    assert!(error.to_string().contains("artifact site does not match"));
    Ok(())
}
