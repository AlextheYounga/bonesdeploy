use std::env;
use std::fs;
use std::process;

use anyhow::Result;

use bonesremote::runtime::docker::command::{APP_PATH, SHARED_PATH, application_command, container_name, image_name};

#[test]
fn names_are_site_scoped() -> Result<()> {
    assert_eq!(image_name("atlas")?, "bonesdeploy/laravel-atlas:runtime");
    assert_eq!(container_name("atlas")?, "bonesdeploy-atlas");
    Ok(())
}

#[test]
fn application_command_has_restricted_mounts() -> Result<()> {
    let root = env::temp_dir().join(format!("bonesremote-docker-command-{}", process::id()));
    fs::create_dir_all(root.join("current"))?;
    fs::create_dir_all(root.join("shared"))?;
    let command = application_command("atlas", &root, "root", "image")?;
    let args = command.get_args().map(|arg| arg.to_string_lossy()).collect::<Vec<_>>().join(" ");
    assert!(args.contains("--read-only"));
    assert!(args.contains("--cap-drop=ALL"));
    assert!(args.contains("no-new-privileges"));
    assert!(args.contains(APP_PATH));
    assert!(args.contains(SHARED_PATH));
    assert!(!args.contains("docker.sock"));
    fs::remove_dir_all(root)?;
    Ok(())
}
