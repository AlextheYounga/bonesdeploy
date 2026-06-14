use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};

use super::{InitArgs, cli_existing_or_prompt, collect_non_interactive, run};

use anyhow::{Result, bail};
use shared::paths;
use tempfile::TempDir;

use crate::config::{BonesConfig, Data, Releases, Ssl};

fn test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct TestEnvironment {
    _lock: std::sync::MutexGuard<'static, ()>,
    original_dir: PathBuf,
    original_home: Option<String>,
}

impl TestEnvironment {
    fn enter(repo_dir: &Path, home_dir: &Path) -> Result<Self> {
        let lock = test_lock().lock().map_err(|_| anyhow::anyhow!("test lock poisoned"))?;
        let original_dir = env::current_dir()?;
        let original_home = env::var("HOME").ok();

        env::set_current_dir(repo_dir)?;

        // Safety: these tests serialize access with a process-wide mutex and restore HOME on drop.
        unsafe {
            env::set_var("HOME", home_dir);
        }

        Ok(Self { _lock: lock, original_dir, original_home })
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.original_dir);

        match &self.original_home {
            Some(home) => {
                // Safety: these tests serialize access with a process-wide mutex and restore HOME on drop.
                unsafe {
                    env::set_var("HOME", home);
                }
            }
            None => {
                // Safety: these tests serialize access with a process-wide mutex and restore HOME on drop.
                unsafe {
                    env::remove_var("HOME");
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

fn incomplete_seed(project_name: &str) -> BonesConfig {
    BonesConfig {
        data: Data {
            remote_name: String::from("production"),
            project_name: String::from(project_name),
            host: String::new(),
            port: String::from("22"),
            repo_path: String::new(),
            project_root: String::new(),
            web_root: String::new(),
            branch: String::from("main"),
            deploy_on_push: true,
            ..Default::default()
        },
        releases: Releases::default(),
        ssl: Ssl::default(),
    }
}

/// Uses seed config and CLI values without prompting when non-interactive mode is active.
#[test]
fn collect_non_interactive_uses_seed_and_cli_values_without_prompting() -> Result<()> {
    let seed = incomplete_seed("atlas");
    let args = InitArgs {
        non_interactive: true,
        setup_remote: false,
        project_name: None,
        branch: None,
        remote: None,
        host: Some(String::from("deploy.example.com")),
        port: None,
    };

    let cfg = collect_non_interactive("workspace", Some(&seed), &args)?;

    assert_eq!(cfg.data.project_name, "atlas");
    assert_eq!(cfg.data.host, "deploy.example.com");
    assert_eq!(cfg.data.branch, "main");
    assert_eq!(cfg.data.remote_name, "production");
    assert_eq!(cfg.data.repo_path, paths::default_repo_path_for("atlas"));

    Ok(())
}

/// Requires a host when neither seed config nor CLI provide one.
#[test]
fn collect_non_interactive_requires_host_when_seed_and_cli_are_missing_it() -> Result<()> {
    let seed = incomplete_seed("atlas");
    let args = InitArgs {
        non_interactive: true,
        setup_remote: false,
        project_name: None,
        branch: None,
        remote: None,
        host: None,
        port: None,
    };

    let result = collect_non_interactive("workspace", Some(&seed), &args);
    let Err(err) = result else {
        bail!("missing host should fail");
    };
    assert!(err.to_string().contains("--host is required"));

    Ok(())
}

/// Reuses an existing project name instead of prompting again when init seeded one already.
#[test]
fn cli_existing_or_prompt_prefers_existing_value_before_prompt() -> Result<()> {
    let value = cli_existing_or_prompt(None, Some(String::from("lawsnipe")), || bail!("prompt should not run"))?;

    assert_eq!(value, "lawsnipe");

    Ok(())
}

/// Materializes the pyinfra deploy files and shared infra assets during init.
#[test]
fn init_materializes_base_infra_assets() -> Result<()> {
    with_temp_repo(|repo_dir, _home_dir| {
        run(&init_args())?;

        let bones_dir = repo_dir.join(".bones");
        assert!(bones_dir.join("infra/setup.py").is_file());
        assert!(bones_dir.join("infra/runtime.py").is_file());
        assert!(bones_dir.join("infra/ssl.py").is_file());
        assert!(bones_dir.join("infra/assets/nginx/router.conf.j2").is_file());

        let config_root = paths::bones_config_root().join("atlas.bones");
        assert!(config_root.join("infra/setup.py").is_file());

        assert!(!bones_dir.join("infra/.venv").exists());
        assert!(!bones_dir.join("infra/__pycache__").exists());
        assert!(!bones_dir.join("infra/.python-version").exists());
        assert!(!bones_dir.join("infra/pyproject.toml").exists());

        Ok(())
    })
}

/// Keeps an already materialized infra scaffold intact when init is run again.
#[test]
fn init_rerun_preserves_existing_infra_assets() -> Result<()> {
    with_temp_repo(|repo_dir, _home_dir| {
        run(&init_args())?;

        let sentinel = repo_dir.join(".bones/infra/setup.py");
        let original = fs::read_to_string(&sentinel)?;

        run(&init_args())?;

        assert!(sentinel.is_file());
        assert_eq!(fs::read_to_string(&sentinel)?, original);

        Ok(())
    })
}
