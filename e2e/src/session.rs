//! Isolated "developer machine" for a test run: a throwaway `HOME` with its
//! own SSH keypair, ssh config, known_hosts, and gitconfig. Nothing from the
//! real user environment leaks in, and container host keys never pollute the
//! real `~/.ssh/known_hosts`.
//!
//! An `ssh` shim is installed at `<home>/bin/ssh` and prepended to PATH.
//! OpenSSH ignores `$HOME` for config discovery (it uses getpwuid), so the
//! shim is the only reliable way to force `-F <session-config>` onto every
//! bare `Command::new("ssh")` call in the process tree.

use std::ffi::OsStr;
use std::fs;
use std::os::unix::fs::PermissionsExt as _;
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

        let bin_dir = home.join("bin");
        fs::create_dir_all(&bin_dir).with_context(|| format!("Failed to create {}", bin_dir.display()))?;
        let shim_path = bin_dir.join("ssh");
        fs::write(&shim_path, "#!/bin/sh\nexec /usr/bin/ssh -F \"$HOME/.ssh/config\" \"$@\"\n")
            .context("Failed to write ssh shim")?;
        fs::set_permissions(&shim_path, fs::Permissions::from_mode(0o755))
            .context("Failed to make ssh shim executable")?;

        Ok(Self { home, keep: keep_artifacts() })
    }

    pub fn home(&self) -> &Path {
        &self.home
    }

    pub fn public_key(&self) -> Result<String> {
        let path = self.home.join(".ssh/id_ed25519.pub");
        fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))
    }

    /// A command wired to this session: fake `HOME`, no ssh-agent, a
    /// persistent `XDG_CONFIG_HOME` so the materialized bonesinfra venv is
    /// cached across runs, and `bin/` prepended to PATH so the session ssh
    /// shim intercepts every bare `ssh` invocation in the process tree.
    pub fn command(&self, program: impl AsRef<OsStr>) -> Command {
        let bin_dir = self.home.join("bin");
        let path = match std::env::var_os("PATH") {
            Some(p) => {
                let mut dirs = std::env::split_paths(&p).collect::<Vec<_>>();
                dirs.insert(0, bin_dir);
                std::env::join_paths(dirs).unwrap_or(p)
            }
            None => bin_dir.into_os_string(),
        };
        let mut command = Command::new(program);
        command
            .env("HOME", &self.home)
            .env("XDG_CONFIG_HOME", scratch_dir().join("xdg-config"))
            .env("PATH", path)
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
