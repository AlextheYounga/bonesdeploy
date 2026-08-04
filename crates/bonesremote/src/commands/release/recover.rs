use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use bonesdeploy_core::config;
use bonesdeploy_core::paths;

use crate::privileges;
use crate::release::state::{self as release_state, DeploymentLock};

pub fn run(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote release recover")?;
    recover_site(site)
}

/// Quarantines malformed `active-deployment.json` state so the site becomes
/// recoverable (status, cancellation, and idle checks all parse the file).
///
/// A running deployment holds the site's deployment lock for its entire
/// lifetime, so acquiring the lock here proves no deployment process is alive:
/// only then is it safe to move malformed state aside without racing the
/// deployer that wrote it.
fn recover_site(site: &str) -> Result<()> {
    config::validate_site_name(site)?;
    let _lock = DeploymentLock::acquire(site)
        .with_context(|| format!("Refusing to recover {site} while a deployment is running"))?;

    let active_path = release_state::active_deployment_path(site);
    if !active_path.exists() {
        println!("No active deployment state for {site}.");
        return Ok(());
    }

    match release_state::read_active_deployment(site) {
        Ok(_) => {
            println!("Active deployment state for {site} is valid; nothing to recover.");
        }
        Err(parse_error) => {
            let quarantine_dir = release_state::recovery_dir(site);
            fs::create_dir_all(&quarantine_dir)
                .with_context(|| format!("Failed to create quarantine directory {}", quarantine_dir.display()))?;
            let stamp = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
            let target = quarantine_dir.join(format!("{}-{stamp}.json", paths::ACTIVE_DEPLOYMENT_FILE));
            fs::rename(&active_path, &target)
                .with_context(|| format!("Failed to quarantine {} into {}", active_path.display(), target.display()))?;
            println!(
                "Quarantined malformed active deployment state for {site} to {}\n  not parseable: {parse_error}",
                target.display()
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use anyhow::Result;
    use bonesdeploy_core::paths;

    use super::recover_site;
    use crate::release::state::{
        self as release_state, ActiveDeployment, DeploymentPhase, ScopedRoot, set_sites_root_for_tests,
    };

    fn temp_root(test_name: &str) -> Result<(ScopedRoot, PathBuf)> {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
        let path = env::temp_dir().join(format!("bonesremote_recover_{test_name}_{}_{}", process::id(), nanos));
        fs::create_dir_all(&path)?;
        Ok((set_sites_root_for_tests(path.clone()), path))
    }

    #[test]
    fn recover_quarantines_malformed_active_deployment_state() -> Result<()> {
        let (_guard, root) = temp_root("malformed")?;
        let site_root = root.join("unitapp");
        fs::create_dir_all(&site_root)?;
        fs::write(site_root.join(paths::ACTIVE_DEPLOYMENT_FILE), "{ not valid json")?;

        recover_site("unitapp")?;

        assert!(!site_root.join(paths::ACTIVE_DEPLOYMENT_FILE).exists());
        let quarantine = site_root.join(paths::RECOVERY_DIR);
        assert!(fs::read_dir(&quarantine)?.next().is_some(), "malformed state must be quarantined");

        fs::remove_dir_all(root).ok();
        Ok(())
    }

    #[test]
    fn recover_is_noop_when_state_is_valid() -> Result<()> {
        let (_guard, root) = temp_root("valid")?;
        let site_root = root.join("unitapp");
        fs::create_dir_all(&site_root)?;
        let deployment = ActiveDeployment {
            release: String::from("20260804_190321-46a0b75c-0000"),
            pid: process::id(),
            process_start_ticks: 0,
            phase: DeploymentPhase::Building,
            started_at: String::from("2026-08-04T19:03:21Z"),
            context: None,
        };
        release_state::write_active_deployment("unitapp", &deployment)?;

        recover_site("unitapp")?;

        assert!(release_state::active_deployment_path("unitapp").exists());
        assert!(!site_root.join(paths::RECOVERY_DIR).exists());

        fs::remove_dir_all(root).ok();
        Ok(())
    }

    #[test]
    fn recover_refuses_while_a_deployment_holds_the_lock() -> Result<()> {
        let (_guard, _root) = temp_root("locked")?;
        let _lock = release_state::DeploymentLock::acquire("unitapp")?;

        assert!(recover_site("unitapp").is_err());
        Ok(())
    }
}
