use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

use crate::bootstrap_ssh;
use crate::config;
use crate::git;
use crate::prompts;
use crate::pyinfra;
use crate::pyinfra::PyinfraDeploy;
use crate::remote_data;

pub fn run() -> Result<()> {
    git::ensure_git_repository()?;

    let bones_dir = Path::new(config::Constants::BONES_DIR);
    if !bones_dir.exists() {
        bail!(".bones/ does not exist. Run `bonesdeploy init` first.");
    }

    let bones_yaml = Path::new(config::Constants::BONES_YAML);
    let cfg = config::load(bones_yaml)?;

    let runtime_yaml = Path::new(config::Constants::BONES_RUNTIME_YAML);
    if !runtime_yaml.exists() {
        bail!("{} does not exist. Run `bonesdeploy init` first.", config::Constants::BONES_RUNTIME_YAML);
    }

    if !prompts::confirm_remote_runtime()? {
        println!("Skipped remote runtime apply.");
        return Ok(());
    }

    let ssh_user = bootstrap_ssh::resolve();
    let deploy_file = PathBuf::from(config::Constants::BONES_REMOTE_RUNTIME_DEPLOY);

    pyinfra::ensure_pyinfra_installed()?;
    let deploy_data = remote_data::runtime(&cfg, runtime_yaml)?;
    pyinfra::run_pyinfra_deploy(
        &cfg,
        &ssh_user,
        &deploy_data,
        &PyinfraDeploy { extra_args: &[], deploy_file: &deploy_file },
    )
}
