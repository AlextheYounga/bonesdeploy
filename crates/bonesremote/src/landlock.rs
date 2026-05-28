use std::path::PathBuf;

use anyhow::Result;

pub struct Policy {
    pub read_only_paths: Vec<PathBuf>,
    #[cfg_attr(not(target_os = "linux"), allow(dead_code))]
    pub writable_paths: Vec<PathBuf>,
}

#[cfg(target_os = "linux")]
mod platform {
    use super::Policy;

    use ::landlock::{
        ABI, Access, AccessFs, CompatLevel, Compatible, LandlockStatus, PathBeneath, PathFd, Ruleset, RulesetStatus,
    };
    use anyhow::{Context, Result, bail};

    pub fn verify_support() -> Result<()> {
        Ruleset::default()
            .set_compatibility(CompatLevel::HardRequirement)
            .handle_access(AccessFs::Execute)
            .context("Landlock ruleset handling is unavailable")?
            .create()
            .context("Landlock ruleset creation failed")?;

        Ok(())
    }

    pub fn restrict_self(policy: &Policy) -> Result<()> {
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

        Ok(())
    }
}

#[cfg(not(target_os = "linux"))]
mod platform {
    use super::Policy;

    use anyhow::{Result, bail};

    pub fn verify_support() -> Result<()> {
        bail!("Landlock is only available on Linux")
    }

    pub fn restrict_self(_policy: &Policy) -> Result<()> {
        bail!("Landlock is only available on Linux")
    }
}

pub fn verify_support() -> Result<()> {
    platform::verify_support()
}

pub fn restrict_self(policy: &Policy) -> Result<()> {
    platform::restrict_self(policy)
}

pub fn default_system_read_paths() -> Vec<PathBuf> {
    ["/usr", "/lib", "/lib64", "/bin", "/sbin", "/etc", "/dev", "/proc"]
        .into_iter()
        .map(PathBuf::from)
        .filter(|path| path.exists())
        .collect()
}
