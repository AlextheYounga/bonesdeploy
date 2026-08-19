use std::{
    env, fs,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;

use bonesremote::release::lifecycle::build::build_user::*;

fn temp_dir(prefix: &str) -> Result<PathBuf> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
    let path = env::temp_dir().join(format!("{prefix}_{nanos}"));
    fs::create_dir_all(&path)?;
    Ok(path)
}

#[test]
fn build_cache_validation_requires_private_owned_directory() -> Result<()> {
    let root = temp_dir("bonesremote-build-cache")?;
    let cache = root.join("cache");
    fs::create_dir(&cache)?;
    fs::set_permissions(&cache, PermissionsExt::from_mode(0o700))?;
    let metadata = fs::metadata(&cache)?;
    validate_build_cache(&cache, metadata.uid(), metadata.gid())?;

    fs::set_permissions(&cache, PermissionsExt::from_mode(0o755))?;
    assert!(validate_build_cache(&cache, metadata.uid(), metadata.gid()).is_err());
    fs::remove_dir_all(root).ok();
    Ok(())
}

fn command_to_string(command: &Command) -> String {
    Command::new("sh")
        .arg("-c")
        .arg("printf '%s\\n' \"$@\"")
        .arg("dummy")
        .args(command.get_args())
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
        .unwrap_or_default()
}

#[test]
fn build_script_command_includes_runtime_max_sec() {
    let command = build_script_command("demo-build", 300);
    assert!(command_to_string(&command).contains("--property=RuntimeMaxSec=300s"));
}

#[test]
fn plain_build_user_command_has_no_runtime_max_sec() {
    let command = build_user_command("demo-build");
    assert!(!command_to_string(&command).contains("RuntimeMaxSec"));
}

#[test]
fn build_script_command_runs_as_the_build_user_machine() {
    let command = build_script_command("demo-build", 300);
    assert!(command_to_string(&command).contains("--machine=demo-build@"));
}
