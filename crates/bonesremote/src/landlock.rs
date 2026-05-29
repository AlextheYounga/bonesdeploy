use std::path::PathBuf;

use anyhow::Result;

pub struct Policy {
    pub read_only_paths: Vec<PathBuf>,
    pub writable_paths: Vec<PathBuf>,
}

fn policy_path_counts(policy: &Policy) -> (usize, usize) {
    (policy.read_only_paths.len(), policy.writable_paths.len())
}

#[cfg(target_os = "linux")]
mod platform {
    use super::Policy;

    use ::landlock::{
        ABI, Access, AccessFs, CompatLevel, Compatible, LandlockStatus, PathBeneath, PathFd, Ruleset, RulesetAttr,
        RulesetCreatedAttr, RulesetStatus,
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

    pub fn restrict_self(policy: &Policy) -> Result<()> {
        let _ = super::policy_path_counts(policy);
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

#[cfg(test)]
mod tests {
    use super::{Policy, policy_path_counts};
    use std::path::PathBuf;

    #[test]
    fn linux_landlock_module_imports_ruleset_attr_trait() {
        let source = include_str!("landlock.rs");
        let production_source = source.split("#[cfg(test)]").next().unwrap_or(source);

        assert!(
            production_source
                .contains("CompatLevel, Compatible, LandlockStatus, PathBeneath, PathFd, Ruleset, RulesetAttr,")
                || production_source.contains(
                    "CompatLevel, Compatible, LandlockStatus, PathBeneath, PathFd, Ruleset, RulesetAttr, RulesetStatus,"
                ),
            "linux landlock module must import RulesetAttr so handle_access compiles with landlock 0.4.x\n{production_source}"
        );
    }

    #[test]
    fn linux_landlock_module_imports_ruleset_created_attr_trait() {
        let source = include_str!("landlock.rs");
        let production_source = source.split("#[cfg(test)]").next().unwrap_or(source);

        assert!(
            production_source.contains("RulesetCreatedAttr") && production_source.contains("set_no_new_privs(true)"),
            "linux landlock module must import RulesetCreatedAttr so set_no_new_privs compiles with landlock 0.4.x\n{production_source}"
        );
    }

    #[test]
    fn policy_path_counts_reports_read_and_write_lengths() {
        let policy = Policy {
            read_only_paths: vec![PathBuf::from("/usr"), PathBuf::from("/etc")],
            writable_paths: vec![PathBuf::from("/run/acme")],
        };

        assert_eq!(policy_path_counts(&policy), (2, 1));
    }
}
