use std::fs;
use std::path::Path;

use anyhow::Result;
use console::style;

mod config;
mod runtime;
mod scaffold;

pub struct Args {
    pub non_interactive: bool,
    pub project_name: Option<String>,
    pub branch: Option<String>,
    pub remote: Option<String>,
    pub host: Option<String>,
    pub port: Option<String>,
    pub template: Option<String>,
    pub runtime_vars: Vec<String>,
}

use crate::config as bones_config;
use crate::infra::git;
use crate::ui::output;
use shared::paths;

#[derive(Debug)]
pub(super) struct RuntimeSelection {
    template: Option<String>,
    config: serde_json::Map<String, serde_json::Value>,
}

pub fn run(args: &Args) -> Result<()> {
    run_with_prefetch(args, || Ok(()))
}

pub(super) fn run_with_prefetch(args: &Args, prefetch_bonesinfra: impl FnOnce() -> Result<()>) -> Result<()> {
    git::ensure_git_repository()?;

    println!("{} {}", style("Initializing").cyan().bold(), style("bonesdeploy").bold());
    prefetch_bonesinfra()?;

    let bones_dir = Path::new(paths::LOCAL_BONES_DIR);
    let had_bones_entry = fs::symlink_metadata(bones_dir).is_ok();
    let is_fresh = !bones_dir.exists();
    if !is_fresh {
        println!("Using existing .bones config.");
    }

    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    let mut cfg =
        if is_fresh { config::collect_fresh_config(args)? } else { config::load_or_collect_config(bones_toml, args)? };
    let runtime_selection =
        if is_fresh { Some(runtime::collect_runtime_config(args, &cfg.project_name)?) } else { None };

    if let Some(runtime) = runtime_selection {
        scaffold::materialize_fresh_bones(bones_dir, had_bones_entry, &mut cfg, runtime)?;
    }

    scaffold::update_gitignore()?;
    scaffold::ensure_config_gitignore(&cfg.project_name)?;
    bones_config::save(&cfg, bones_toml)?;

    if is_fresh {
        println!("{} bonesdeploy initialized.", output::success_marker());
    } else {
        println!("{} bonesdeploy config updated.", output::success_marker());
    }

    scaffold::ensure_local_remote(&cfg)?;
    scaffold::install_pre_push_guard()?;
    print_follow_up_hint();

    Ok(())
}

fn print_follow_up_hint() {
    println!();
    println!("{}", output::next_step_with_detail("bonesdeploy setup", "to setup the remote server"));
}

#[cfg(test)]
mod tests {
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

    fn init_args() -> Args {
        Args {
            non_interactive: true,
            project_name: Some(String::from("atlas")),
            branch: None,
            remote: None,
            host: Some(String::from("deploy.example.com")),
            port: None,
            template: None,
            runtime_vars: Vec::new(),
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
            assert!(!bones_dir.join("hooks").exists(), ".bones should not contain a hooks/ directory");
            let deploy_dir = bones_dir.join("deployment");
            assert!(deploy_dir.is_dir());
            assert!(deploy_dir.read_dir()?.next().is_some(), "deployment directory should have scripts");
            let bones_toml = fs::read_to_string(bones_dir.join("bones.toml"))?;
            assert!(bones_toml.contains("runtime_user = \"atlas\""));
            assert!(bones_toml.contains("[runtime]"));

            let pre_push = repo_dir.join(".git/hooks/pre-push");
            assert!(pre_push.is_file(), "guaranteed pre-push guard should be installed");
            let guard_content = fs::read_to_string(&pre_push)?;
            assert!(guard_content.contains("bonesdeploy-pre-push-v1"));

            let config_root = paths::bones_config_root().join("atlas.bones");
            assert!(!config_root.join("hooks").exists(), "config hooks/ should not be created");

            let config_gitignore = paths::bones_config_root().join(".gitignore");
            assert!(config_gitignore.is_file());
            let gitignore_content = fs::read_to_string(config_gitignore)?;
            assert!(gitignore_content.contains("_lib"));
            assert!(gitignore_content.contains("atlas.bones"));

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
            assert!(!paths::bones_config_root().join("atlas.bones").exists());

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
            assert_eq!(fs::read_link(&bones_dir)?, paths::bones_config_root().join("atlas.bones"));

            Ok(())
        })
    }
}
