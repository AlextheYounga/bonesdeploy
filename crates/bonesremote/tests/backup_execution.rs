use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Result, ensure};
use bonesdeploy_core::config::Bones;
use bonesremote::release::SiteMutation;
use bonesremote::release::state::{DeploymentLock, override_sites_root};
use bonesremote::shared::backup::BackupJob;
use time::macros::datetime;

#[test]
fn backup_creates_shared_archive_and_prunes_with_age_based_retention() -> Result<()> {
    let root = tempfile::tempdir()?;
    let bin = root.path().join("bin");
    fs::create_dir(&bin)?;
    let borg = bin.join("borg");
    fs::write(
        &borg,
        "#!/bin/sh\nprintf '%s\\n' --call-- \"$@\" >> \"$BONES_BORG_ARGS\"\nprintf '%s\\n' \"$BORG_PASSPHRASE\" >> \"$BONES_BORG_SECRETS\"\ncase \" $* \" in *' create '*) printf '%s' \"$BONES_BORG_CREATE_JSON\";; esac\n",
    )?;
    fs::set_permissions(&borg, fs::Permissions::from_mode(0o755))?;
    let json = serde_json::json!({
        "repository": {"id":"id", "location":"repo", "last_modified":"2026-08-28T12:34:56"},
        "archive": {"id":"id", "name":"archive", "command_line":[], "limits":{"max_archive_size":0.0},
            "duration":0.0, "start":"2026-08-28T12:34:56", "end":"2026-08-28T12:34:56",
            "stats":{"compressed_size":0,"deduplicated_size":0,"nfiles":0,"original_size":0}}
    });
    let output = Command::new(env::current_exe()?)
        .args(["--exact", "run_backup_with_recording_borg", "--nocapture"])
        .env("BONES_BACKUP_TEST_ROOT", root.path())
        .env("BONES_BORG_ARGS", root.path().join("arguments"))
        .env("BONES_BORG_SECRETS", root.path().join("secrets"))
        .env("BONES_BORG_CREATE_JSON", json.to_string())
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
fn run_backup_with_recording_borg() -> Result<()> {
    let Some(root) = env::var_os("BONES_BACKUP_TEST_ROOT") else { return Ok(()) };
    let root = PathBuf::from(root);
    let repository = root.join("atlas.borg");
    let shared = root.join("shared");
    let passphrase = root.join("passphrase");
    fs::create_dir(&shared)?;
    let _sites = override_sites_root(root.join("sites"));
    let mutation = SiteMutation::adopt("atlas", Bones::for_site("atlas"), DeploymentLock::acquire("atlas")?)?;
    let job = BackupJob { repository: &repository, shared: &shared, passphrase_file: &passphrase, keep_days: 30 };
    let now = datetime!(2026-08-28 12:34:56 UTC);
    let Err(missing) = job.run(&mutation, now) else { anyhow::bail!("missing passphrase must fail") };
    assert!(missing.to_string().contains("Missing Borg passphrase"));
    fs::write(&passphrase, "\n")?;
    let Err(empty) = job.run(&mutation, now) else { anyhow::bail!("empty passphrase must fail") };
    assert!(empty.to_string().contains("is empty"));
    assert!(!root.join("arguments").exists());
    fs::write(&passphrase, "secret value\n")?;
    job.run(&mutation, now)?;
    let arguments = fs::read_to_string(root.join("arguments"))?;
    let calls = arguments.split("--call--\n").filter(|part| !part.is_empty()).collect::<Vec<_>>();
    assert_eq!(calls.len(), 2, "{arguments}");
    assert!(calls[0].lines().any(|arg| arg == "create"));
    assert!(calls[0].contains(&format!("{}::atlas_20260828_123456", repository.display())));
    assert!(calls[0].lines().any(|arg| arg == shared.to_string_lossy()));
    assert!(calls[1].lines().any(|arg| arg == "prune"));
    assert!(calls[1].lines().any(|arg| arg == repository.to_string_lossy()));
    assert!(calls[1].lines().any(|arg| arg == "--keep-within"));
    assert!(calls[1].lines().any(|arg| arg == "30d"));
    assert!(!calls[1].contains("--keep-daily"));
    assert!(!arguments.contains("secret value"));
    assert_eq!(fs::read_to_string(root.join("secrets"))?, "secret value\nsecret value\n");
    Ok(())
}
