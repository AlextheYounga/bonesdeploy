use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process;

use anyhow::Result;

use crate::commands::ensure_site_idle;
use crate::release::state::{self as release_state, DeploymentLock};

use super::{reject_plaintext_env_files, validate_repo_path, write_hook_file};
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
fn plaintext_env_validation_allows_local_files() -> Result<()> {
    let root = env::temp_dir().join(format!("bonesremote-site-buildtime-test-{}", process::id()));
    if root.exists() {
        fs::remove_dir_all(&root)?;
    }
    fs::create_dir_all(&root)?;
    fs::write(root.join("custom.py"), "print('local')\n")?;
    fs::create_dir_all(root.join("__pycache__"))?;

    let result = reject_plaintext_env_files(&root);
    fs::remove_dir_all(&root)?;
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn plaintext_env_validation_rejects_nested_env_files() -> Result<()> {
    let root = env::temp_dir().join(format!("bonesremote-site-test-{}", process::id()));
    if root.exists() {
        fs::remove_dir_all(&root)?;
    }
    fs::create_dir_all(&root)?;
    fs::create_dir_all(root.join("secrets"))?;
    fs::write(root.join("secrets/.env"), "PASSWORD=secret\n")?;

    let result = reject_plaintext_env_files(&root);
    fs::remove_dir_all(&root)?;
    assert!(result.is_err());
    Ok(())
}

#[test]
fn plaintext_env_validation_allows_encrypted_env_files() -> Result<()> {
    let root = env::temp_dir().join(format!("bonesremote-site-confs-test-{}", process::id()));
    if root.exists() {
        fs::remove_dir_all(&root)?;
    }
    fs::create_dir_all(&root)?;
    fs::write(root.join(".env.gpg"), "encrypted\n")?;

    let result = reject_plaintext_env_files(&root);
    fs::remove_dir_all(&root)?;
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn arbitrary_deployment_files_are_allowed() -> Result<()> {
    let root = env::temp_dir().join(format!("bonesremote-site-deployment-test-{}", process::id()));
    if root.exists() {
        fs::remove_dir_all(&root)?;
    }
    fs::create_dir_all(root.join("deployment/build"))?;
    fs::create_dir_all(root.join("deployment/prepare"))?;
    fs::write(root.join("deployment/functions.sh"), "#!/bin/bash\n")?;
    fs::write(root.join("deployment/build/01_build.sh"), "#!/bin/bash\n")?;
    fs::write(root.join("deployment/prepare/02_prepare.sh"), "#!/bin/bash\n")?;

    let result = reject_plaintext_env_files(&root);
    fs::remove_dir_all(&root)?;
    assert!(result.is_ok());
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
