use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process;

use anyhow::Result;

use crate::commands::ensure_site_idle;
use crate::release::state::{self as release_state, DeploymentLock};

use super::{validate_deployment_entries, validate_repo_path, validate_top_level_entries, write_hook_file};
use shared::paths;

#[test]
fn imports_share_a_stable_lock_and_reject_staged_releases() -> Result<()> {
    let root = env::temp_dir().join(format!("bonesremote-site-lock-test-{}", process::id()));
    if root.exists() {
        fs::remove_dir_all(&root)?;
    }
    fs::create_dir_all(root.join("unitapp"))?;
    let _guard = release_state::set_sites_root_for_tests(root.clone());

    let lock = DeploymentLock::acquire("unitapp")?;
    fs::rename(root.join("unitapp"), root.join("unitapp.backup"))?;
    fs::create_dir_all(root.join("unitapp"))?;
    assert!(DeploymentLock::acquire("unitapp").is_err());
    drop(lock);

    release_state::write_staged_release("unitapp", "20260507_151501")?;
    assert!(ensure_site_idle("unitapp").is_err());
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn validate_top_level_entries_allows_single_config() -> Result<()> {
    let root = env::temp_dir().join(format!("bonesremote-site-buildtime-test-{}", process::id()));
    if root.exists() {
        fs::remove_dir_all(&root)?;
    }
    fs::create_dir_all(&root)?;
    fs::write(root.join(paths::BONES_TOML), "")?;
    fs::create_dir_all(root.join(paths::DEPLOYMENT_DIR))?;

    let result = validate_top_level_entries(&root);
    fs::remove_dir_all(&root)?;
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn validate_top_level_entries_rejects_unexpected_file() -> Result<()> {
    let root = env::temp_dir().join(format!("bonesremote-site-test-{}", process::id()));
    if root.exists() {
        fs::remove_dir_all(&root)?;
    }
    fs::create_dir_all(&root)?;
    fs::write(root.join("oops.txt"), "bad")?;

    let result = validate_top_level_entries(&root);
    fs::remove_dir_all(&root)?;
    assert!(result.is_err());
    Ok(())
}

#[test]
fn validate_top_level_entries_allows_confs_directory() -> Result<()> {
    let root = env::temp_dir().join(format!("bonesremote-site-confs-test-{}", process::id()));
    if root.exists() {
        fs::remove_dir_all(&root)?;
    }
    fs::create_dir_all(&root)?;
    fs::write(root.join(paths::BONES_TOML), "")?;
    fs::create_dir_all(root.join(paths::DEPLOYMENT_DIR))?;
    fs::create_dir_all(root.join(paths::CONFS_DIR))?;

    let result = validate_top_level_entries(&root);
    fs::remove_dir_all(&root)?;
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn validate_deployment_entries_allows_only_direct_numbered_shell_scripts() -> Result<()> {
    let root = env::temp_dir().join(format!("bonesremote-site-deployment-test-{}", process::id()));
    if root.exists() {
        fs::remove_dir_all(&root)?;
    }
    fs::create_dir_all(root.join("deployment/build"))?;
    fs::create_dir_all(root.join("deployment/prepare"))?;
    fs::write(root.join("deployment/functions.sh"), "#!/bin/bash\n")?;
    fs::write(root.join("deployment/build/01_build.sh"), "#!/bin/bash\n")?;
    fs::write(root.join("deployment/prepare/02_prepare.sh"), "#!/bin/bash\n")?;

    let result = validate_deployment_entries(&root);
    fs::remove_dir_all(&root)?;
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn validate_deployment_entries_rejects_nested_and_unlisted_files() -> Result<()> {
    let root = env::temp_dir().join(format!("bonesremote-site-deployment-reject-test-{}", process::id()));
    if root.exists() {
        fs::remove_dir_all(&root)?;
    }
    fs::create_dir_all(root.join("deployment/build/nested"))?;
    fs::write(root.join("deployment/build/README.md"), "not a script\n")?;

    let result = validate_deployment_entries(&root);
    fs::remove_dir_all(&root)?;
    assert!(result.is_err());
    Ok(())
}

#[test]
fn write_hook_file_installs_baked_trigger_with_executable_mode() -> Result<()> {
    let root = env::temp_dir().join(format!("bonesremote-site-hook-test-{}", process::id()));
    if root.exists() {
        fs::remove_dir_all(&root)?;
    }

    let repo_root = root.join("repos/unitapp.git");
    let target = repo_root.join(paths::HOOKS_DIR).join("post-receive");

    write_hook_file(&target)?;

    let contents = fs::read_to_string(&target)?;
    let mode = fs::metadata(&target)?.permissions().mode() & 0o777;

    assert!(contents.contains("bonesdeploy-post-receive-v1"));
    assert_eq!(mode, 0o755);
    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn validate_repo_path_rejects_paths_outside_configured_parent() {
    assert!(validate_repo_path("/home/git/unitapp.git", "unitapp").is_ok());
    assert!(validate_repo_path("/home/git/../etc/passwd", "unitapp").is_err());
    assert!(validate_repo_path("/home/git/other.git", "unitapp").is_err());
    assert!(validate_repo_path("/srv/repos/unitapp.git", "unitapp").is_err());
    assert!(validate_repo_path("relative/unitapp.git", "unitapp").is_err());
    assert!(validate_repo_path("/home/git/unitapp.git/", "unitapp").is_err());
    assert!(validate_repo_path("", "unitapp").is_err());
    assert!(validate_repo_path("/home/git/unitapp.git", "other").is_err());
}
