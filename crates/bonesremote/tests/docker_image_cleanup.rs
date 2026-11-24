use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Result, ensure};
use bonesdeploy_core::artifact::COMPOSE_IMAGE_INVENTORY_FILE;
use bonesremote::runtime::docker::command::{remove_unreferenced_image_tags, remove_unreferenced_release_images};

#[test]
fn cleanup_removes_unretained_images_but_preserves_shared_and_absent_tags() -> Result<()> {
    let root = tempfile::tempdir()?;
    let bin = root.path().join("bin");
    fs::create_dir(&bin)?;
    let docker = bin.join("docker");
    fs::write(
        &docker,
        "#!/bin/sh\nprintf '%s %s %s\\n' \"$1\" \"$2\" \"$3\" >> \"$BONES_DOCKER_LOG\"\ncase \"$2:$3\" in rm:already-absent|inspect:already-absent|rm:cannot-remove) exit 1;; esac\n",
    )?;
    fs::set_permissions(&docker, fs::Permissions::from_mode(0o755))?;
    let output = Command::new(env::current_exe()?)
        .args(["--exact", "run_cleanup_with_recording_docker", "--nocapture"])
        .env("BONES_CLEANUP_ROOT", root.path())
        .env("BONES_DOCKER_LOG", root.path().join("docker.log"))
        .env("PATH", bin)
        .output()?;
    ensure!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

#[test]
fn run_cleanup_with_recording_docker() -> Result<()> {
    let Some(root) = env::var_os("BONES_CLEANUP_ROOT") else { return Ok(()) };
    let root = PathBuf::from(root);
    let discarded = root.join("discarded");
    let retained = root.join("retained");
    fs::create_dir(&discarded)?;
    fs::create_dir(&retained)?;
    fs::write(
        discarded.join(COMPOSE_IMAGE_INVENTORY_FILE),
        serde_json::to_vec(&[
            serde_json::json!({"service":"web", "tag":"pre-promotion"}),
            serde_json::json!({"service":"job", "tag":"shared"}),
            serde_json::json!({"service":"worker", "tag":"already-absent"}),
        ])?,
    )?;
    fs::write(
        retained.join(COMPOSE_IMAGE_INVENTORY_FILE),
        serde_json::to_vec(&[serde_json::json!({"service":"job", "tag":"shared"})])?,
    )?;

    remove_unreferenced_release_images(&discarded, &[retained])?;
    let log = fs::read_to_string(root.join("docker.log"))?;
    assert_eq!(
        log.lines().collect::<Vec<_>>(),
        ["image rm pre-promotion", "image rm already-absent", "image inspect already-absent"]
    );
    let Err(error) = remove_unreferenced_image_tags(&["cannot-remove".into()], &[]) else {
        anyhow::bail!("cleanup must fail when an image remains after removal failed");
    };
    assert!(error.to_string().contains("cannot-remove"));
    Ok(())
}
