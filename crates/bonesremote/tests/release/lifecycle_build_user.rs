use std::{
    env, fs,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::PathBuf,
    process::Command,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{Error, Result};

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
    let command = build_script_command("demo-build", "bonesdeploy-build-demo-script.service", 300);
    assert!(command_to_string(&command).contains("--property=RuntimeMaxSec=300s"));
}

#[test]
fn plain_build_user_command_has_no_runtime_max_sec() {
    let command = build_user_command("demo-build");
    assert!(!command_to_string(&command).contains("RuntimeMaxSec"));
}

#[test]
fn build_script_command_runs_as_the_build_user_machine() {
    let command = build_script_command("demo-build", "bonesdeploy-build-demo-script.service", 300);
    assert!(command_to_string(&command).contains("--machine=demo-build@"));
}

#[test]
fn timed_build_script_uses_an_inspectable_named_unit_without_collection() {
    let unit = build_script_unit("demo", 1);
    let command = build_script_command("demo-build", &unit, 300);
    let arguments = command_to_string(&command);
    assert!(arguments.contains("--unit\nbonesdeploy-build-demo-script-1.service"));
    assert!(!arguments.contains("--collect"));
}

#[test]
fn numbered_build_scripts_receive_distinct_transient_units() {
    assert_ne!(build_script_unit("demo", 1), build_script_unit("demo", 2));
}

#[test]
fn only_systemd_timeout_results_trigger_timeout_containment() {
    assert!(build_script_timed_out("timeout"));
    assert!(!build_script_timed_out("exit-code"));
    assert!(!build_script_timed_out("signal"));
}

#[test]
fn containment_errors_remain_identifiable_through_context() {
    let error = Error::new(BuildContainmentError).context("cgroup remained populated");
    assert!(is_build_containment_error(&error));
}

#[test]
fn build_user_cgroup_termination_targets_uid_slice_and_accepts_empty_cgroup() -> Result<()> {
    let root = temp_dir("bonesremote-cgroup-empty")?;
    let slice = root.join("user.slice/user-1234.slice");
    fs::create_dir_all(&slice)?;
    fs::write(slice.join("cgroup.kill"), "")?;
    fs::write(slice.join("cgroup.events"), "populated 0\nfrozen 0\n")?;
    let cgroup = BuildUserCgroup::new(&root, 1234, Duration::ZERO);

    let mut stopped_uid = None;
    cgroup.terminate_with(|uid| {
        stopped_uid = Some(uid);
        Ok(())
    })?;

    assert_eq!(cgroup.path(), slice);
    assert_eq!(fs::read_to_string(cgroup.path().join("cgroup.kill"))?, "1");
    assert_eq!(stopped_uid, Some(1234));
    fs::remove_dir_all(root).ok();
    Ok(())
}

#[test]
fn build_user_cgroup_termination_fails_without_kill_control() -> Result<()> {
    let root = temp_dir("bonesremote-cgroup-no-kill")?;
    let slice = root.join("user.slice/user-1234.slice");
    fs::create_dir_all(&slice)?;
    fs::write(slice.join("cgroup.events"), "populated 0\n")?;
    let cgroup = BuildUserCgroup::new(&root, 1234, Duration::ZERO);

    assert!(cgroup.terminate_with(|_| Ok(())).is_err());
    fs::remove_dir_all(root).ok();
    Ok(())
}

#[test]
fn build_user_cgroup_termination_fails_without_verification_control() -> Result<()> {
    let root = temp_dir("bonesremote-cgroup-no-events")?;
    let slice = root.join("user.slice/user-1234.slice");
    fs::create_dir_all(&slice)?;
    fs::write(slice.join("cgroup.kill"), "")?;
    let cgroup = BuildUserCgroup::new(&root, 1234, Duration::ZERO);

    assert!(cgroup.terminate_with(|_| Ok(())).is_err());
    fs::remove_dir_all(root).ok();
    Ok(())
}

#[test]
fn build_user_cgroup_termination_fails_when_slice_remains_populated() -> Result<()> {
    let root = temp_dir("bonesremote-cgroup-populated")?;
    let slice = root.join("user.slice/user-1234.slice");
    fs::create_dir_all(&slice)?;
    fs::write(slice.join("cgroup.kill"), "")?;
    fs::write(slice.join("cgroup.events"), "populated 1\n")?;
    let cgroup = BuildUserCgroup::new(&root, 1234, Duration::ZERO);

    let error = cgroup
        .terminate_with(|_| Ok(()))
        .err()
        .ok_or_else(|| anyhow::anyhow!("populated cgroup did not fail closed"))?;
    assert!(error.to_string().contains("remained populated"));
    fs::remove_dir_all(root).ok();
    Ok(())
}
