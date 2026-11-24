mod common;

use std::fs;
use std::net::{TcpListener, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Child, Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Result, bail, ensure};
use bonesdeploy::infra::ssh::shell_quote;
use bonesdeploy_core::paths;
use common::TestEnv;

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
    let fixture = ExportSshd::start(env.repo())?;
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

    let output = fixture.export(&env, "complete.zip")?;
    ensure!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let complete = env.repo().join("complete.zip");
    assert_eq!(fs::read(&complete)?, b"archive bytes");
    assert_eq!(fs::metadata(&complete)?.permissions().mode() & 0o777, 0o600);
    let command = fs::read_to_string(env.repo().join("remote-command"))?;
    let root = paths::default_project_root_for("atlas");
    let remote = format!("cd {} && exec zip -q -r -y - shared", shell_quote(&root));
    assert_eq!(command, format!("bash -c {}", shell_quote(&remote)));

    fs::write(env.repo().join("race-enabled"), "")?;
    let output = fixture.export(&env, "race.zip")?;
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

struct ExportSshd {
    directory: tempfile::TempDir,
    process: Child,
    port: u16,
}

impl ExportSshd {
    fn start(project: &Path) -> Result<Self> {
        let directory = tempfile::tempdir()?;
        let key = directory.path().join("identity");
        let host_key = directory.path().join("host-key");
        for path in [&key, &host_key] {
            ensure!(
                Command::new("ssh-keygen").args(["-q", "-t", "ed25519", "-N", "", "-f"]).arg(path).status()?.success()
            );
        }
        let authorized = directory.path().join("authorized_keys");
        fs::copy(key.with_extension("pub"), &authorized)?;
        let port = TcpListener::bind("127.0.0.1:0")?.local_addr()?.port();
        let known_hosts = directory.path().join("known_hosts");
        fs::write(&known_hosts, format!("[127.0.0.1]:{port} {}", fs::read_to_string(host_key.with_extension("pub"))?))?;
        let server_script = directory.path().join("export-command");
        let quote = shell_quote;
        fs::write(
            &server_script,
            format!(
                "#!/bin/sh\nprintf '%s' \"$SSH_ORIGINAL_COMMAND\" > {}\nif [ -e {} ]; then printf '%s' 'existing archive' > {}; fi\nprintf '%s' 'archive bytes'\n",
                quote(&project.join("remote-command").display().to_string()),
                quote(&project.join("race-enabled").display().to_string()),
                quote(&project.join("race.zip").display().to_string())
            ),
        )?;
        fs::set_permissions(&server_script, fs::Permissions::from_mode(0o755))?;
        let config = directory.path().join("sshd_config");
        fs::write(
            &config,
            format!(
                "Port {port}\nListenAddress 127.0.0.1\nHostKey {}\nAuthorizedKeysFile {}\nPidFile {}\nPasswordAuthentication no\nChallengeResponseAuthentication no\nUsePAM no\nStrictModes no\nLogLevel ERROR\nForceCommand {}\n",
                host_key.display(),
                authorized.display(),
                directory.path().join("sshd.pid").display(),
                server_script.display()
            ),
        )?;
        let process =
            Command::new("/usr/bin/sshd").args(["-D", "-e", "-f"]).arg(config).stderr(Stdio::null()).spawn()?;
        let fixture = Self { directory, process, port };
        let deadline = Instant::now() + Duration::from_secs(5);
        while TcpStream::connect(("127.0.0.1", port)).is_err() {
            if Instant::now() >= deadline {
                bail!("fixture sshd did not start");
            }
            thread::sleep(Duration::from_millis(10));
        }
        let ssh = fixture.directory.path().join("ssh");
        fs::write(
            &ssh,
            format!(
                "#!/bin/sh\nexec /usr/bin/ssh -i {} -o UserKnownHostsFile={} \"$@\"\n",
                quote(&key.display().to_string()),
                quote(&known_hosts.display().to_string())
            ),
        )?;
        fs::set_permissions(ssh, fs::Permissions::from_mode(0o755))?;
        Ok(fixture)
    }

    fn export(&self, env: &TestEnv, filename: &str) -> Result<Output> {
        let path = format!("{}:/usr/bin:/bin", self.directory.path().display());
        Ok(Command::new(common::binary())
            .args(["site", "export", "--output", filename])
            .current_dir(env.repo())
            .env("HOME", env.home())
            .env("XDG_CONFIG_HOME", env.home().join(".config"))
            .env("XDG_DATA_HOME", env.home().join(".local/share"))
            .env("XDG_STATE_HOME", env.home().join(".local/state"))
            .env("PATH", path)
            .output()?)
    }
}

impl Drop for ExportSshd {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}
