use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use ::landlock::{
    ABI, Access, AccessFs, CompatLevel, Compatible, LandlockStatus, PathBeneath, PathFd, RestrictionStatus, Ruleset,
    RulesetAttr, RulesetCreatedAttr, RulesetStatus,
};
use anyhow::{Context, Result, bail};

pub struct Policy {
    pub read_only_paths: Vec<PathBuf>,
    pub writable_paths: Vec<PathBuf>,
}

pub fn verify_support() -> Result<()> {
    Ruleset::default()
        .set_compatibility(CompatLevel::HardRequirement)
        .handle_access(AccessFs::Execute)
        .context("Landlock ruleset handling is unavailable")?
        .create()
        .context("Landlock ruleset creation failed")?;

    Ok(())
}

pub fn restrict_self(policy: &Policy) -> Result<RestrictionStatus> {
    let abi = ABI::V6;
    let read_access = AccessFs::from_read(abi) | AccessFs::Execute;
    let write_access = AccessFs::from_all(abi);

    let mut ruleset = Ruleset::default()
        .set_compatibility(CompatLevel::BestEffort)
        .handle_access(read_access)
        .context("Failed to configure Landlock read access")?
        .handle_access(write_access)
        .context("Failed to configure Landlock write access")?
        .create()
        .context("Failed to create Landlock ruleset")?
        .set_no_new_privs(true);

    for path in &policy.read_only_paths {
        ruleset = ruleset
            .add_rule(PathBeneath::new(PathFd::new(path)?, read_access))
            .with_context(|| format!("Failed to add read-only Landlock rule for {}", path.display()))?;
    }

    for path in &policy.writable_paths {
        ruleset = ruleset
            .add_rule(PathBeneath::new(PathFd::new(path)?, write_access))
            .with_context(|| format!("Failed to add writable Landlock rule for {}", path.display()))?;
    }

    let status = ruleset.restrict_self().context("Failed to apply Landlock restrictions")?;

    if status.ruleset == RulesetStatus::NotEnforced {
        bail!("Landlock ruleset was not enforced");
    }

    if matches!(status.landlock, LandlockStatus::NotEnabled | LandlockStatus::NotImplemented) {
        bail!("Landlock is not available on this host");
    }

    Ok(status)
}

pub fn resolve_command_path(command: &str) -> Result<PathBuf> {
    let command_path = Path::new(command);
    if command_path.is_absolute() || command_path.components().count() > 1 {
        if !command_path.exists() {
            bail!("Runtime command does not exist: {}", command_path.display());
        }
        return fs::canonicalize(command_path)
            .with_context(|| format!("Failed to resolve runtime command path: {}", command_path.display()));
    }

    let path_env = env::var_os("PATH").ok_or_else(|| anyhow::anyhow!("PATH is not set"))?;
    for dir in env::split_paths(&path_env) {
        let candidate = dir.join(command);
        if candidate.is_file() {
            return fs::canonicalize(&candidate)
                .with_context(|| format!("Failed to resolve runtime command path: {}", candidate.display()));
        }
    }

    bail!("Runtime command is not available in PATH: {command}")
}

pub fn default_system_read_paths() -> Vec<PathBuf> {
    ["/usr", "/lib", "/lib64", "/bin", "/sbin", "/etc", "/dev", "/proc"]
        .into_iter()
        .map(PathBuf::from)
        .filter(|path| path.exists())
        .collect()
}
