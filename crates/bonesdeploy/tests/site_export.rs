mod common;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

use anyhow::{Result, ensure};
use bonesdeploy::infra::ssh::shell_quote;
use bonesdeploy_core::paths;
use common::{TestEnv, ssh::SshFixture};

#[test]
fn export_refuses_an_existing_archive_without_modifying_it() -> Result<()> {
    let env = TestEnv::new()?;
    let init =
        env.run(&["init", "--non-interactive", "--project-name", "atlas", "--host", "127.0.0.1", "--port", "1"])?;
    assert!(init.status.success(), "{}", String::from_utf8_lossy(&init.stderr));
    let destination = env.repo().join("existing.zip");
    fs::write(&destination, b"existing archive")?;

    let output = env.run(&["site", "export", "--output", "existing.zip"])?;
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Refusing to overwrite existing export"));
    assert_eq!(fs::read(destination)?, b"existing archive");
    Ok(())
}

#[test]
fn failed_export_removes_its_temporary_archive() -> Result<()> {
    let env = TestEnv::new()?;
    let init =
        env.run(&["init", "--non-interactive", "--project-name", "atlas", "--host", "127.0.0.1", "--port", "1"])?;
    assert!(init.status.success(), "{}", String::from_utf8_lossy(&init.stderr));
    let destination = env.repo().join("exports");
    fs::create_dir(&destination)?;

    let output = env.run(&["site", "export", "--output", "exports/archive.zip"])?;
    assert!(!output.status.success());
    assert_eq!(fs::read_dir(destination)?.count(), 0, "failed transfer left an archive or staging file");
    Ok(())
}

#[test]
fn export_publishes_privately_and_preserves_destination_created_during_transfer() -> Result<()> {
    let env = TestEnv::new()?;
    let quote = shell_quote;
    let project = env.repo();
    let script = format!(
        "#!/bin/sh\nprintf '%s' \"$SSH_ORIGINAL_COMMAND\" > {}\nif [ -e {} ]; then printf '%s' 'existing archive' > {}; fi\nprintf '%s' 'archive bytes'\n",
        quote(&project.join("remote-command").display().to_string()),
        quote(&project.join("race-enabled").display().to_string()),
        quote(&project.join("race.zip").display().to_string())
    );
    let fixture = SshFixture::start(&script)?;
    let init = env.run(&[
        "init",
        "--non-interactive",
        "--project-name",
        "atlas",
        "--host",
        "127.0.0.1",
        "--port",
        &fixture.port.to_string(),
    ])?;
    ensure!(init.status.success(), "{}", String::from_utf8_lossy(&init.stderr));
    let dotenv = env.repo().join(".env");
    let username = String::from_utf8(Command::new("id").arg("-un").output()?.stdout)?.trim().to_owned();
    let contents = fs::read_to_string(&dotenv)?.replace("SSH_USER=root", &format!("SSH_USER={username}"));
    fs::write(dotenv, contents)?;

    let output = fixture.run(&env, &["site", "export", "--output", "complete.zip"])?;
    ensure!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let complete = env.repo().join("complete.zip");
    assert_eq!(fs::read(&complete)?, b"archive bytes");
    assert_eq!(fs::metadata(&complete)?.permissions().mode() & 0o777, 0o600);
    let command = fs::read_to_string(env.repo().join("remote-command"))?;
    let root = paths::default_project_root_for("atlas");
    let remote = format!("cd {} && exec zip -q -r -y - shared", shell_quote(&root));
    assert_eq!(command, format!("bash -c {}", shell_quote(&remote)));

    fs::write(env.repo().join("race-enabled"), "")?;
    let output = fixture.run(&env, &["site", "export", "--output", "race.zip"])?;
    assert!(!output.status.success(), "destination created during transfer must not be replaced");
    assert!(String::from_utf8_lossy(&output.stderr).contains("Failed to save shared export"));
    assert_eq!(fs::read(env.repo().join("race.zip"))?, b"existing archive");
    assert_eq!(
        fs::read_dir(env.repo())?
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().starts_with(".tmp"))
            .count(),
        0
    );
    Ok(())
}
