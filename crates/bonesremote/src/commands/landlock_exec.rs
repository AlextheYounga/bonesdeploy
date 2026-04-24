use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::config;
use crate::landlock;
use crate::privileges;
use crate::release_state;

pub fn run(config_path: &str) -> Result<()> {
    privileges::ensure_not_root("bonesremote landlock exec")?;

    let cfg = config::load(Path::new(config_path))?;
    if cfg.runtime.command.is_empty() {
        bail!("runtime.command must be configured before starting runtime with landlock");
    }

    let active_runtime_root = fs::canonicalize(&cfg.data.live_root)
        .with_context(|| format!("Failed to resolve live_root: {}", cfg.data.live_root))?;
    let shared_root = release_state::shared_dir(&cfg);
    let command_path = landlock::resolve_command_path(&cfg.runtime.command[0])?;

    let working_dir = resolve_working_dir(&cfg.runtime.working_dir, &active_runtime_root)?;
    let policy = build_policy(&cfg, &active_runtime_root, &shared_root, &command_path)?;

    privileges::set_no_new_privs()?;
    landlock::restrict_self(&policy)?;

    env::set_current_dir(&working_dir)
        .with_context(|| format!("Failed to change working directory to {}", working_dir.display()))?;

    let mut command = Command::new(&cfg.runtime.command[0]);
    command.args(cfg.runtime.command.iter().skip(1));

    let exec_error = command.exec();
    bail!("Failed to exec runtime command {:?}: {exec_error}", cfg.runtime.command)
}

fn resolve_working_dir(working_dir: &str, runtime_root: &Path) -> Result<PathBuf> {
    let candidate =
        if Path::new(working_dir).is_absolute() { PathBuf::from(working_dir) } else { runtime_root.join(working_dir) };

    let resolved = fs::canonicalize(&candidate)
        .with_context(|| format!("Failed to resolve runtime working_dir {}", candidate.display()))?;

    if !resolved.starts_with(runtime_root) {
        bail!("runtime.working_dir resolves outside active runtime root: {}", resolved.display());
    }

    Ok(resolved)
}

fn build_policy(
    cfg: &config::BonesConfig,
    runtime_root: &Path,
    shared_root: &Path,
    command_path: &Path,
) -> Result<landlock::Policy> {
    let mut read_only_paths = BTreeSet::new();
    read_only_paths.insert(runtime_root.to_path_buf());

    if let Some(parent) = command_path.parent() {
        read_only_paths.insert(parent.to_path_buf());
    }

    for system_path in landlock::default_system_read_paths() {
        read_only_paths.insert(system_path);
    }

    let mut writable_paths = BTreeSet::new();

    writable_paths.insert(runtime_root.to_path_buf());

    if shared_root.exists() {
        let resolved_shared_root = fs::canonicalize(shared_root)
            .with_context(|| format!("Failed to resolve shared root {}", shared_root.display()))?;
        writable_paths.insert(resolved_shared_root);
    }

    for additional_root in &cfg.runtime.writable_paths {
        writable_paths.insert(resolve_additional_writable_root(additional_root, runtime_root)?);
    }

    Ok(landlock::Policy {
        read_only_paths: read_only_paths.into_iter().collect(),
        writable_paths: writable_paths.into_iter().collect(),
    })
}

fn resolve_additional_writable_root(path: &str, runtime_root: &Path) -> Result<PathBuf> {
    let candidate = if Path::new(path).is_absolute() { PathBuf::from(path) } else { runtime_root.join(path) };
    fs::canonicalize(&candidate).with_context(|| format!("Failed to resolve writable root {}", candidate.display()))
}
