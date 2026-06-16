use std::path::Path;

use anyhow::{Result, bail};

use crate::config;
use crate::embedded;
use crate::git;
use crate::prompts;
use crate::python;

pub fn run() -> Result<()> {
    git::ensure_git_repository()?;

    let bones_dir = Path::new(config::Constants::BONES_DIR);
    if !bones_dir.exists() {
        bail!(".bones/ does not exist. Run `bonesdeploy init` first.");
    }

    embedded::ensure_infra_assets_exist(bones_dir)?;

    let runtime_toml = Path::new(config::Constants::BONES_RUNTIME_TOML);
    if !runtime_toml.exists() {
        bail!("{} does not exist. Run `bonesdeploy init` first.", config::Constants::BONES_RUNTIME_TOML);
    }

    if !prompts::confirm_remote_runtime()? {
        println!("Skipped remote runtime apply.");
        return Ok(());
    }

    let bones_toml = Path::new(config::Constants::BONES_TOML);
    println!(
        "Applying runtime using {} ...",
        config::Constants::BONES_INFRA_MAIN
    );

    python::run_python(&[
        "runtime-apply", "apply",
        "--config", bones_toml.to_str().unwrap_or(".bones/bones.toml"),
        "--runtime-config", runtime_toml.to_str().unwrap_or(".bones/runtime.toml"),
    ])?;

    println!("Runtime apply completed.");
    Ok(())
}
