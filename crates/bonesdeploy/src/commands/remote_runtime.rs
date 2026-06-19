use std::path::Path;

use anyhow::{Result, bail};

use crate::git;
use crate::prompts;
use crate::python;
use shared::paths;

pub fn run() -> Result<()> {
    git::ensure_git_repository()?;

    let runtime_toml = Path::new(paths::LOCAL_BONES_RUNTIME_TOML);
    if !runtime_toml.exists() {
        bail!("{} does not exist. Run `bonesdeploy init` first.", paths::LOCAL_BONES_RUNTIME_TOML);
    }

    if !prompts::confirm_remote_runtime()? {
        println!("Skipped remote runtime apply.");
        return Ok(());
    }

    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    println!("Applying runtime using hidden bonesinfra ...");

    python::run(&[
        "runtime",
        "apply",
        "--config",
        bones_toml.to_str().unwrap_or(".bones/bones.toml"),
        "--runtime-config",
        runtime_toml.to_str().unwrap_or(".bones/runtime.toml"),
    ])?;

    println!("Runtime apply completed.");
    Ok(())
}
