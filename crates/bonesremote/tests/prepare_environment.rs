use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Result, ensure};
use bonesremote::release::lifecycle::prepare::PrepareScript;

#[test]
fn prepare_script_receives_generic_site_context_without_framework_version_projection() -> Result<()> {
    let root = tempfile::tempdir()?;
    let bin = root.path().join("bin");
    fs::create_dir(&bin)?;
    let runuser = bin.join("runuser");
    fs::write(&runuser, "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$BONES_TEST_RUNUSER_ARGS\"\nexec /bin/bash -s\n")?;
    fs::set_permissions(&runuser, fs::Permissions::from_mode(0o755))?;
    let output = Command::new(env::current_exe()?)
        .args(["--exact", "run_prepare_with_recording_runuser", "--nocapture"])
        .env("BONES_PREPARE_TEST_ROOT", root.path())
        .env("BONES_TEST_RUNUSER_ARGS", root.path().join("arguments"))
        .env("PATH", bin)
        .env_remove("BONES_RUNTIME_RUBY_VERSION")
        .env_remove("BONES_RUNTIME_PYTHON_VERSION")
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
fn run_prepare_with_recording_runuser() -> Result<()> {
    let Some(root) = env::var_os("BONES_PREPARE_TEST_ROOT") else { return Ok(()) };
    let root = PathBuf::from(root);
    let functions = root.join("functions.sh");
    let script = root.join("01_prepare.sh");
    let log = root.join("prepare.log");
    let release = root.join("release");
    fs::create_dir(&release)?;
    fs::write(&functions, "#!/bin/bash\n")?;
    fs::write(
        &script,
        "printf '%s\\n' \"$PROJECT_NAME\" \"$PROJECT_ROOT\" \"$REPO_PATH\" \"$WEB_ROOT\" \"$SERVICE_USER\" \"${BONES_RUNTIME_RUBY_VERSION+set}\" \"${BONES_RUNTIME_PYTHON_VERSION+set}\"\n",
    )?;
    let prepare = PrepareScript {
        project_name: "atlas",
        project_root: "/srv/sites/atlas",
        runtime_user: "atlas",
        web_root: "public",
        shared_functions: &functions,
    };
    assert!(prepare.run(&script, &release, &log)?.success());
    assert_eq!(fs::read_to_string(log)?, "atlas\n/srv/sites/atlas\n\npublic\natlas\n\n\n");
    assert_eq!(fs::read_to_string(root.join("arguments"))?, "-u\natlas\n--\nbash\n-c\numask 0002; exec bash -s\n");
    Ok(())
}
