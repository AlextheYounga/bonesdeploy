//! Isolated "developer machine" for a test run: a throwaway `HOME` with its
//! own SSH keypair, ssh config, known_hosts, and gitconfig. Nothing from the
//! real user environment leaks in, and container host keys never pollute the
//! real `~/.ssh/known_hosts`.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};

use crate::{keep_artifacts, scratch_dir, status_ok, unique_suffix};

pub struct Session {
    home: PathBuf,
    keep: bool,
}

impl Session {
    pub fn create() -> Result<Self> {
        let home = scratch_dir().join(format!("home-{}", unique_suffix()));
        let ssh_dir = home.join(".ssh");
        fs::create_dir_all(&ssh_dir).with_context(|| format!("Failed to create {}", ssh_dir.display()))?;

        let keygen = Command::new("ssh-keygen")
            .args(["-q", "-t", "ed25519", "-N", "", "-C", "bones-e2e", "-f"])
            .arg(ssh_dir.join("id_ed25519"))
            .status()
            .context("Failed to run ssh-keygen")?;
        status_ok(keygen, "ssh-keygen")?;

        let ssh_config = format!(
            "Host *\n\
             \x20 StrictHostKeyChecking accept-new\n\
             \x20 UserKnownHostsFile {known_hosts}\n\
             \x20 IdentityFile {identity}\n\
             \x20 IdentitiesOnly yes\n",
            known_hosts = ssh_dir.join("known_hosts").display(),
            identity = ssh_dir.join("id_ed25519").display(),
        );
        fs::write(ssh_dir.join("config"), ssh_config).context("Failed to write ssh config")?;

        let gitconfig = "[user]\n\tname = Bones E2E\n\temail = e2e@bonesdeploy.test\n[init]\n\tdefaultBranch = main\n";
        fs::write(home.join(".gitconfig"), gitconfig).context("Failed to write .gitconfig")?;

        Ok(Self { home, keep: keep_artifacts() })
    }

    pub fn home(&self) -> &Path {
        &self.home
    }

    pub fn public_key(&self) -> Result<String> {
        let path = self.home.join(".ssh/id_ed25519.pub");
        fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))
    }

    /// A command wired to this session: fake `HOME`, no ssh-agent, and a
    /// persistent `XDG_CONFIG_HOME` so the materialized bonesinfra venv is
    /// cached across runs instead of rebuilt every time.
    pub fn command(&self, program: impl AsRef<OsStr>) -> Command {
        let mut command = Command::new(program);
        command
            .env("HOME", &self.home)
            .env("XDG_CONFIG_HOME", scratch_dir().join("xdg-config"))
            .env_remove("SSH_AUTH_SOCK");
        command
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        if self.keep {
            eprintln!("{}: keeping session home {} for inspection", crate::KEEP_ENV, self.home.display());
            return;
        }
        if let Err(err) = fs::remove_dir_all(&self.home) {
            eprintln!("Failed to clean up session home {}: {err}", self.home.display());
        }
    }
}
