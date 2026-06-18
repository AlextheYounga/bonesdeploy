use std::path::Path;

use anyhow::{Result, bail};

use crate::bootstrap_ssh;
use crate::config;
use crate::git;
use crate::prompts;
use crate::python;

pub fn run() -> Result<()> {
    git::ensure_git_repository()?;

    let runtime_toml = Path::new(config::Constants::BONES_RUNTIME_TOML);
    if !runtime_toml.exists() {
        bail!("{} does not exist. Run `bonesdeploy init` first.", config::Constants::BONES_RUNTIME_TOML);
    }

    if !prompts::confirm_remote_runtime()? {
        println!("Skipped remote runtime apply.");
        return Ok(());
    }

    let bones_toml = Path::new(config::Constants::BONES_TOML);
    let cfg = config::load(bones_toml)?;
    let ssh_user = bootstrap_ssh::resolve(Some(&cfg.ssh_user));
    println!("Applying runtime using hidden bonesinfra ...");

    python::run(&[
        "runtime",
        "apply",
        "--config",
        bones_toml.to_str().unwrap_or(".bones/bones.toml"),
        "--runtime-config",
        runtime_toml.to_str().unwrap_or(".bones/runtime.toml"),
        "--ssh-user",
        &ssh_user,
    ])?;

    println!("Runtime apply completed.");
    Ok(())
}
