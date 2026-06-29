use std::env;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, MutexGuard, OnceLock};

use super::{collect_non_interactive, run_with_prefetch};
use crate::commands::init_config::InitArgs;

use anyhow::{Result, bail};
use shared::paths;
use tempfile::TempDir;

use crate::config::Bones;

fn test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct TestEnvironment {
    _lock: MutexGuard<'static, ()>,
    original_dir: PathBuf,
    original_home: Option<String>,
    original_xdg_config_home: Option<String>,
}

impl TestEnvironment {
    fn enter(repo_dir: &Path, home_dir: &Path) -> Result<Self> {
        let lock = test_lock().lock().map_err(|_| anyhow::anyhow!("test lock poisoned"))?;
        let original_dir = env::current_dir()?;
        let original_home = env::var("HOME").ok();
        let original_xdg_config_home = env::var("XDG_CONFIG_HOME").ok();

        env::set_current_dir(repo_dir)?;

        // Safety: these tests serialize access with a process-wide mutex and restore env vars on drop.
        unsafe {
            env::set_var("HOME", home_dir);
            env::set_var("XDG_CONFIG_HOME", home_dir.join(".config"));
        }

        Ok(Self { _lock: lock, original_dir, original_home, original_xdg_config_home })
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.original_dir);

        match &self.original_home {
            Some(home) => {
                // Safety: these tests serialize access with a process-wide mutex and restore env vars on drop.
                unsafe {
                    env::set_var("HOME", home);
                }
            }
            None => {
                // Safety: these tests serialize access with a process-wide mutex and restore env vars on drop.
                unsafe {
                    env::remove_var("HOME");
                }
            }
        }

        match &self.original_xdg_config_home {
            Some(home) => {
                // Safety: these tests serialize access with a process-wide mutex and restore env vars on drop.
                unsafe {
                    env::set_var("XDG_CONFIG_HOME", home);
                }
            }
            None => {
                // Safety: these tests serialize access with a process-wide mutex and restore env vars on drop.
                unsafe {
                    env::remove_var("XDG_CONFIG_HOME");
                }
            }
        }
    }
}

fn init_args() -> InitArgs {
    InitArgs {
        non_interactive: true,
        setup_remote: false,
        project_name: Some(String::from("atlas")),
        branch: None,
        remote: None,
        host: Some(String::from("deploy.example.com")),
        port: None,
    }
}

fn run_init() -> Result<bool> {
    run_with_prefetch(&init_args(), || Ok(()))
}

fn create_git_repo(path: &Path) -> Result<()> {
    let status = Command::new("git").args(["init", "--quiet"]).current_dir(path).status()?;
    if !status.success() {
        bail!("git init failed with status {status}");
    }
    Ok(())
}

fn with_temp_repo(test: impl FnOnce(&Path, &Path) -> Result<()>) -> Result<()> {
    let temp = TempDir::new()?;
    let repo_dir = temp.path().join("repo");
    let home_dir = temp.path().join("home");

    fs::create_dir_all(&repo_dir)?;
    fs::create_dir_all(&home_dir)?;
    create_git_repo(&repo_dir)?;

    let _environment = TestEnvironment::enter(&repo_dir, &home_dir)?;

    test(&repo_dir, &home_dir)
}

fn incomplete_existing(project_name: &str) -> Bones {
    Bones {
        remote_name: String::from("production"),
        project_name: String::from(project_name),
        host: String::new(),
        port: String::from("22"),
        repo_path: String::new(),
        project_root: String::new(),
        branch: String::from("main"),
        deploy_on_push: true,
        ..Default::default()
    }
}

/// Uses existing config and CLI values without prompting when non-interactive mode is active.
#[test]
fn collect_non_interactive_uses_existing_and_cli_values_without_prompting() -> Result<()> {
    let existing = incomplete_existing("atlas");
    let args = InitArgs {
        non_interactive: true,
        setup_remote: false,
        project_name: None,
        branch: None,
        remote: None,
        host: Some(String::from("deploy.example.com")),
        port: None,
    };

    let cfg = collect_non_interactive("workspace", Some(&existing), &args)?;

    assert_eq!(cfg.project_name, "atlas");
    assert_eq!(cfg.host, "deploy.example.com");
    assert_eq!(cfg.branch, "main");
    assert_eq!(cfg.remote_name, "production");
    assert_eq!(cfg.repo_path, paths::default_repo_path_for("atlas"));

    Ok(())
}

/// Requires a host when neither existing config nor CLI provide one.
#[test]
fn collect_non_interactive_requires_host_when_existing_and_cli_are_missing_it() -> Result<()> {
    let existing = incomplete_existing("atlas");
    let args = InitArgs {
        non_interactive: true,
        setup_remote: false,
        project_name: None,
        branch: None,
        remote: None,
        host: None,
        port: None,
    };

    let result = collect_non_interactive("workspace", Some(&existing), &args);
    let Err(err) = result else {
        bail!("missing host should fail");
    };
    assert!(err.to_string().contains("--host is required"));

    Ok(())
}

/// Materializes the base bonesdeploy kit and runtime config during init.
#[test]
fn init_materializes_base_bones_assets() -> Result<()> {
    with_temp_repo(|repo_dir, _home_dir| {
        run_init()?;

        let bones_dir = repo_dir.join(".bones");
        assert!(bones_dir.join("bones.toml").is_file());
        assert!(bones_dir.join("runtime.toml").is_file());
        assert!(bones_dir.join("hooks/pre-push").is_file());
        assert!(bones_dir.join("hooks/post-receive").is_file());
        let deploy_dir = bones_dir.join("deployment");
        assert!(deploy_dir.is_dir());
        assert!(deploy_dir.read_dir()?.next().is_some(), "deployment directory should have scripts");
        let runtime_toml = fs::read_to_string(bones_dir.join("runtime.toml"))?;
        assert!(runtime_toml.contains("runtime_user = \"atlas\""));
        assert!(runtime_toml.contains("permissions"));

        let config_root = paths::bones_config_root().join("atlas.bones");
        assert!(config_root.join("hooks/pre-push").is_file());
        assert!(config_root.join("hooks/post-receive").is_file());

        let config_gitignore = paths::bones_config_root().join(".gitignore");
        assert!(config_gitignore.is_file());
        let gitignore_content = fs::read_to_string(config_gitignore)?;
        assert!(gitignore_content.contains("gnupg"));
        assert!(gitignore_content.contains("atlas.bones"));

        Ok(())
    })
}

/// Keeps an already materialized local bones scaffold intact when init is run again.
#[test]
fn init_rerun_preserves_existing_bones_assets() -> Result<()> {
    with_temp_repo(|repo_dir, _home_dir| {
        run_init()?;

        let sentinel = repo_dir.join(".bones/hooks/pre-push");
        let original = fs::read_to_string(&sentinel)?;

        run_init()?;

        assert!(sentinel.is_file());
        assert_eq!(fs::read_to_string(&sentinel)?, original);

        Ok(())
    })
}

/// Repairs a dangling .bones symlink instead of failing with EEXIST.
#[test]
fn init_repairs_dangling_bones_symlink() -> Result<()> {
    with_temp_repo(|repo_dir, home_dir| {
        let config_root = home_dir.join(".config/bonesdeploy");
        fs::create_dir_all(&config_root)?;
        symlink(config_root.join("missing.bones"), repo_dir.join(".bones"))?;

        run_init()?;

        let bones_dir = repo_dir.join(".bones");
        assert!(bones_dir.join("bones.toml").is_file());
        assert_eq!(fs::read_link(&bones_dir)?, paths::bones_config_root().join("atlas.bones"));

        Ok(())
    })
}
