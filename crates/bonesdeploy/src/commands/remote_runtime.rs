use std::path::Path;

use anyhow::{Result, bail};

use crate::config;
use crate::git;
use crate::prompts;
use crate::python;

pub fn run() -> Result<()> {
    git::ensure_git_repository()?;

    let bones_dir = Path::new(config::Constants::BONES_DIR);
    if !bones_dir.exists() {
        bail!(".bones/ does not exist. Run `bonesdeploy init` first.");
    }

    let runtime_yaml = Path::new(config::Constants::BONES_RUNTIME_YAML);
    if !runtime_yaml.exists() {
        bail!("{} does not exist. Run `bonesdeploy init` first.", config::Constants::BONES_RUNTIME_YAML);
    }

    if !prompts::confirm_remote_runtime()? {
        println!("Skipped remote runtime apply.");
        return Ok(());
    }

    let bones_yaml = Path::new(config::Constants::BONES_YAML);
    println!(
        "Applying runtime using {} ...",
        config::Constants::BONES_INFRA_MAIN
    );

    python::run_python(&[
        "runtime-apply", "apply",
        "--config", bones_yaml.to_str().unwrap_or(".bones/bones.yaml"),
        "--runtime-config", runtime_yaml.to_str().unwrap_or(".bones/runtime.yaml"),
    ])?;

    println!("Runtime apply completed.");
    Ok(())
}
