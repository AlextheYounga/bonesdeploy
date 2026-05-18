use std::env;
use std::ffi::OsStr;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use anyhow::Result;
use tempfile::TempDir;

pub struct FakeCommandBin {
    _temp_dir: TempDir,
    path: std::ffi::OsString,
    ansible_log: PathBuf,
    ssh_log: PathBuf,
}

impl FakeCommandBin {
    pub fn with_ansible_playbook_and_ssh() -> Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        let ansible_log = temp_dir.path().join("ansible.log");
        let ssh_log = temp_dir.path().join("ssh.log");

        let ansible_playbook = temp_dir.path().join("ansible-playbook");
        let ssh = temp_dir.path().join("ssh");
        fs::write(
            &ansible_playbook,
            format!(
                "#!/usr/bin/env bash\nif [ \"$1\" = \"--version\" ]; then exit 0; fi\necho \"$@\" >{}\nexit 0\n",
                ansible_log.display()
            ),
        )?;
        fs::set_permissions(&ansible_playbook, fs::Permissions::from_mode(0o755))?;
        fs::write(&ssh, format!("#!/usr/bin/env bash\ncat >/dev/null\necho \"$@\" >{}\nexit 0\n", ssh_log.display()))?;
        fs::set_permissions(&ssh, fs::Permissions::from_mode(0o755))?;

        let path = env::join_paths(
            [temp_dir.path().to_path_buf()]
                .into_iter()
                .chain(env::split_paths(&env::var_os("PATH").unwrap_or_default())),
        )?;

        Ok(Self { _temp_dir: temp_dir, path, ansible_log, ssh_log })
    }

    pub fn path(&self) -> &OsStr {
        &self.path
    }

    pub fn ansible_invocation(&self) -> Result<String> {
        Ok(fs::read_to_string(&self.ansible_log)?)
    }

    pub fn ssh_invocation(&self) -> Result<String> {
        Ok(fs::read_to_string(&self.ssh_log)?)
    }
}
