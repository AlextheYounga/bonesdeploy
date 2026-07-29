use std::env;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, MutexGuard, OnceLock};

use anyhow::{Result, bail};
use shared::paths;
use tempfile::TempDir;

use super::{Args, run_with_prefetch};

fn test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct TestEnvironment {
    _lock: MutexGuard<'static, ()>,
    original_dir: PathBuf,
    original_home: Option<String>,
    original_xdg_config_home: Option<String>,
    original_xdg_data_home: Option<String>,
    original_xdg_cache_home: Option<String>,
}

impl TestEnvironment {
    fn enter(repo_dir: &Path, home_dir: &Path) -> Result<Self> {
        let lock = test_lock().lock().map_err(|_| anyhow::anyhow!("test lock poisoned"))?;
        let original_dir = env::current_dir()?;
        let original_home = env::var("HOME").ok();
        let original_xdg_config_home = env::var("XDG_CONFIG_HOME").ok();
        let original_xdg_data_home = env::var("XDG_DATA_HOME").ok();
        let original_xdg_cache_home = env::var("XDG_CACHE_HOME").ok();

        env::set_current_dir(repo_dir)?;

        // Safety: these tests serialize access with a process-wide mutex and restore env vars on drop.
        unsafe {
            env::set_var("HOME", home_dir);
            env::set_var("XDG_CONFIG_HOME", home_dir.join(".config"));
            env::set_var("XDG_DATA_HOME", home_dir.join(".local/share"));
            env::set_var("XDG_CACHE_HOME", home_dir.join(".cache"));
        }

        Ok(Self {
            _lock: lock,
            original_dir,
            original_home,
            original_xdg_config_home,
            original_xdg_data_home,
            original_xdg_cache_home,
        })
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.original_dir);
        restore_env_var("HOME", &self.original_home);
        restore_env_var("XDG_CONFIG_HOME", &self.original_xdg_config_home);
        restore_env_var("XDG_DATA_HOME", &self.original_xdg_data_home);
        restore_env_var("XDG_CACHE_HOME", &self.original_xdg_cache_home);
    }
}

fn restore_env_var(name: &str, original: &Option<String>) {
    // Safety: TestEnvironment holds the test mutex for its entire lifetime; see enter().
    unsafe {
        match original {
            Some(value) => env::set_var(name, value),
            None => env::remove_var(name),
        }
    }
}

fn init_args() -> Args {
    Args {
        non_interactive: true,
        project_name: Some(String::from("atlas")),
        branch: None,
        remote: None,
        host: Some(String::from("deploy.example.com")),
        port: None,
        template: None,
        framework_vars: Vec::new(),
        services: Vec::new(),
    }
}

fn run_init() -> Result<()> {
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

#[test]
fn materializes_base_bones_assets() -> Result<()> {
    with_temp_repo(|repo_dir, _home_dir| {
        run_init()?;

        let bones_dir = repo_dir.join(".bones");
        assert!(bones_dir.join("bones.toml").is_file());
        assert!(bones_dir.join("bones.toml").is_file());
        let custom = fs::read_to_string(bones_dir.join("custom.py"))?;
        assert!(custom.contains("Local-only BonesInfra extension hooks"));
        assert!(!bones_dir.join("hooks").exists(), ".bones should not contain a hooks/ directory");
        let deploy_dir = bones_dir.join("deployment");
        assert!(deploy_dir.is_dir());
        assert!(deploy_dir.read_dir()?.next().is_some(), "deployment directory should have scripts");
        let bones_toml = fs::read_to_string(bones_dir.join("bones.toml"))?;
        assert!(bones_toml.contains("[framework]"));

        let env_build = repo_dir.join(".env.build");
        assert!(env_build.is_file(), ".env.build should be created");
        let env_build_content = fs::read_to_string(&env_build)?;
        assert!(env_build_content.contains("Do not place passwords"));

        let gitignore = fs::read_to_string(repo_dir.join(".gitignore"))?;
        assert!(gitignore.lines().any(|line| line.trim() == "!.env.build"));

        let pre_push = repo_dir.join(".git/hooks/pre-push");
        assert!(pre_push.is_file(), "guaranteed pre-push guard should be installed");
        let guard_content = fs::read_to_string(&pre_push)?;
        assert!(guard_content.contains("bonesdeploy-pre-push-v1"));

        let config_root = paths::bones_projects_root().join("atlas.bones");
        assert!(!config_root.join("hooks").exists(), "config hooks/ should not be created");

        let config_gitignore = paths::bones_config_root().join(".gitignore");
        assert!(config_gitignore.is_file());
        let gitignore_content = fs::read_to_string(config_gitignore)?;
        assert!(gitignore_content.contains("projects/"));

        Ok(())
    })
}

#[test]
fn rerun_preserves_existing_bones_assets() -> Result<()> {
    with_temp_repo(|repo_dir, _home_dir| {
        run_init()?;

        let sentinel = repo_dir.join(".bones/bones.toml");
        let original = fs::read_to_string(&sentinel)?;

        run_init()?;

        assert!(sentinel.is_file());
        assert_eq!(fs::read_to_string(&sentinel)?, original);

        Ok(())
    })
}

#[test]
fn init_preserves_existing_env_build() -> Result<()> {
    with_temp_repo(|repo_dir, _home_dir| {
        let env_build = repo_dir.join(".env.build");
        fs::write(&env_build, "CUSTOM_VAR=custom_value\n")?;

        run_init()?;

        assert_eq!(fs::read_to_string(&env_build)?, "CUSTOM_VAR=custom_value\n");

        Ok(())
    })
}

#[test]
fn failure_before_completed_prompts_leaves_no_bones_assets() -> Result<()> {
    with_temp_repo(|repo_dir, _home_dir| {
        let mut args = init_args();
        args.host = None;

        let result = run_with_prefetch(&args, || Ok(()));
        let Err(err) = result else {
            bail!("init without host should fail");
        };
        assert!(err.to_string().contains("--host is required"));
        assert!(!repo_dir.join(".bones").exists());
        assert!(!paths::bones_projects_root().join("atlas.bones").exists());

        Ok(())
    })
}

#[test]
fn repairs_dangling_bones_symlink() -> Result<()> {
    with_temp_repo(|repo_dir, home_dir| {
        let config_root = home_dir.join(".config/bonesdeploy");
        fs::create_dir_all(&config_root)?;
        symlink(config_root.join("missing.bones"), repo_dir.join(".bones"))?;

        run_init()?;

        let bones_dir = repo_dir.join(".bones");
        assert!(bones_dir.join("bones.toml").is_file());
        assert_eq!(fs::read_link(&bones_dir)?, paths::bones_projects_root().join("atlas.bones"));

        Ok(())
    })
}
