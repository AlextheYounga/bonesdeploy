use std::fs;
use std::net::{TcpListener, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::process::{Child, Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Result, bail, ensure};
use bonesdeploy::infra::ssh::shell_quote;

use super::TestEnv;

pub struct SshFixture {
    directory: tempfile::TempDir,
    process: Child,
    pub port: u16,
}

impl SshFixture {
    pub fn start(script: &str) -> Result<Self> {
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
        let server_script = directory.path().join("remote-command");
        fs::write(&server_script, script)?;
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
                shell_quote(&key.display().to_string()),
                shell_quote(&known_hosts.display().to_string())
            ),
        )?;
        fs::set_permissions(ssh, fs::Permissions::from_mode(0o755))?;
        Ok(fixture)
    }

    pub fn run(&self, env: &TestEnv, args: &[&str]) -> Result<Output> {
        let path = format!("{}:/usr/bin:/bin", self.directory.path().display());
        Ok(Command::new(super::binary())
            .args(args)
            .current_dir(env.repo())
            .env("HOME", env.home())
            .env("XDG_CONFIG_HOME", env.home().join(".config"))
            .env("XDG_DATA_HOME", env.home().join(".local/share"))
            .env("XDG_STATE_HOME", env.home().join(".local/state"))
            .env("PATH", path)
            .output()?)
    }
}

impl Drop for SshFixture {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}
