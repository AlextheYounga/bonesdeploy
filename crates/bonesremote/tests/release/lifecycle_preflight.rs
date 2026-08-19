use std::cell::Cell;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use bonesdeploy_core::config::Bones;

use bonesremote::release::SiteMutation;
use bonesremote::release::lifecycle::preflight::{site_nginx_config, validate_ready};
use bonesremote::release::state::release_dir;
use bonesremote::release::state::{DeploymentLock, override_sites_root};

fn temp_root(test_name: &str) -> Result<PathBuf> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
    let path = env::temp_dir().join(format!("bonesremote_preflight_{}_{}_{}", process::id(), nanos, test_name));
    fs::create_dir_all(&path)?;
    Ok(path)
}

/// Builds a mutation guard whose config points at a temp project root and
/// materializes the release's web root on disk.
fn mutation_for(root: &PathBuf, release: &str) -> Result<SiteMutation> {
    let _guard = override_sites_root(root.join("sites"));
    let lock = DeploymentLock::acquire("unitapp")?;
    let mut config = Bones::default();
    config.app.project_root = root.join("project").to_string_lossy().into_owned();
    config.runtime.web_root = String::from("public");
    let release_dir = release_dir(&config.app.project_root, release);
    fs::create_dir_all(release_dir.join("public"))?;
    Ok(SiteMutation::adopt("unitapp", config, lock))
}

#[test]
fn validate_ready_passes_when_web_root_exists_and_nginx_is_ok() -> Result<()> {
    let root = temp_root("ok")?;
    let mutation = mutation_for(&root, "20260101_000001")?;

    validate_ready(&mutation, "20260101_000001", || Ok(()))?;

    fs::remove_dir_all(root).ok();
    Ok(())
}

#[test]
fn validate_ready_rejects_missing_web_root_before_running_nginx() -> Result<()> {
    let root = temp_root("missing_web_root")?;
    let mutation = mutation_for(&root, "20260101_000002")?;
    fs::remove_dir_all(release_dir(&mutation.config().project_root, "20260101_000002"))?;

    let nginx_called = Cell::new(false);
    let result = validate_ready(&mutation, "20260101_000002", || {
        nginx_called.set(true);
        Ok(())
    });

    assert!(result.is_err());
    assert!(!nginx_called.get(), "nginx -t must not run when the web root is missing");
    fs::remove_dir_all(root).ok();
    Ok(())
}

#[test]
fn validate_ready_propagates_nginx_failure() -> Result<()> {
    let root = temp_root("nginx_fail")?;
    let mutation = mutation_for(&root, "20260101_000003")?;

    let result = validate_ready(&mutation, "20260101_000003", || anyhow::bail!("config broken"));

    assert!(result.is_err());
    let message = result.err().map(|error| error.to_string()).ok_or_else(|| anyhow::anyhow!("expected failure"))?;
    assert!(message.contains("config broken"));
    fs::remove_dir_all(root).ok();
    Ok(())
}

#[test]
fn site_nginx_config_uses_the_registered_sites_configuration() {
    assert_eq!(site_nginx_config("unitapp"), PathBuf::from("/srv/conf/unitapp/nginx.conf"));
}
