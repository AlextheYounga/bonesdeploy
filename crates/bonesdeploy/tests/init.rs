mod common;

use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use common::TestEnv;

const INIT_ARGS: &[&str] = &["init", "--non-interactive", "--project-name", "atlas", "--host", "deploy.example.com"];

fn init_success(env: &TestEnv) -> Result<()> {
    let output = env.run(INIT_ARGS)?;
    assert!(output.status.success(), "init failed: {}", String::from_utf8_lossy(&output.stderr));
    Ok(())
}

fn assert_bones_dir(bones_dir: &Path) -> Result<()> {
    assert!(bones_dir.join("bones.toml").is_file());
    let custom = fs::read_to_string(bones_dir.join("custom.py")).context("read custom.py")?;
    assert!(custom.contains("Local-only BonesInfra extension hooks"));
    assert!(!bones_dir.join("hooks").exists(), ".bones should not contain a hooks/ directory");
    let deploy_dir = bones_dir.join("deployment");
    assert!(deploy_dir.is_dir());
    assert!(deploy_dir.read_dir()?.next().is_some(), "deployment directory should have scripts");
    let bones_toml = fs::read_to_string(bones_dir.join("bones.toml"))?;
    assert!(bones_toml.contains("[runtime]"));
    Ok(())
}

fn atlas_config_root(env: &TestEnv) -> PathBuf {
    env.home().join(".config/bonesdeploy/projects").join("atlas.bones")
}

#[test]
fn materializes_base_bones_assets() -> Result<()> {
    let env = TestEnv::new()?;
    let repo = env.repo();
    init_success(&env)?;

    assert_bones_dir(&repo.join(".bones"))?;

    let env_build = repo.join(".env.build");
    assert!(env_build.is_file(), ".env.build should be created");
    let env_build_content = fs::read_to_string(&env_build)?;
    assert!(env_build_content.contains("Do not place passwords"));

    let gitignore = fs::read_to_string(repo.join(".gitignore"))?;
    assert!(gitignore.lines().any(|line| line.trim() == "!.env.build"));

    let bones_gitignore = fs::read_to_string(repo.join(".bones/.gitignore"))?;
    assert!(bones_gitignore.lines().any(|line| line.trim() == ".env"));
    assert!(bones_gitignore.lines().any(|line| line.trim() == "__pycache__/"));

    let pre_push = repo.join(".git/hooks/pre-push");
    assert!(pre_push.is_file(), "guaranteed pre-push guard should be installed");
    let guard_content = fs::read_to_string(&pre_push)?;
    assert!(guard_content.contains("bonesdeploy-pre-push-v1"));

    assert!(!atlas_config_root(&env).join("hooks").exists(), "config hooks/ should not be created");

    let config_gitignore = env.home().join(".config/bonesdeploy/.gitignore");
    assert!(config_gitignore.is_file());
    let gitignore_content = fs::read_to_string(config_gitignore)?;
    assert!(gitignore_content.contains("projects/"));

    Ok(())
}

#[test]
fn rerun_preserves_existing_bones_assets() -> Result<()> {
    let env = TestEnv::new()?;
    let sentinel = env.repo().join(".bones/bones.toml");

    init_success(&env)?;
    let original = fs::read_to_string(&sentinel)?;

    init_success(&env)?;
    assert!(sentinel.is_file());
    assert_eq!(fs::read_to_string(&sentinel)?, original);

    Ok(())
}

#[test]
fn init_preserves_existing_env_build() -> Result<()> {
    let env = TestEnv::new()?;
    let env_build = env.repo().join(".env.build");
    fs::write(&env_build, "CUSTOM_VAR=custom_value\n")?;

    init_success(&env)?;
    assert_eq!(fs::read_to_string(&env_build)?, "CUSTOM_VAR=custom_value\n");

    Ok(())
}

#[test]
fn failure_before_completed_prompts_leaves_no_bones_assets() -> Result<()> {
    let env = TestEnv::new()?;
    let repo = env.repo();

    let output = env.run(&["init", "--non-interactive", "--project-name", "atlas"])?;

    assert!(!output.status.success(), "init without host should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--host is required"));
    assert!(!repo.join(".bones").exists());
    assert!(!atlas_config_root(&env).exists());

    Ok(())
}

#[test]
fn repairs_dangling_bones_symlink() -> Result<()> {
    let env = TestEnv::new()?;
    let repo = env.repo();
    let home = env.home();

    let config_root = home.join(".config/bonesdeploy");
    fs::create_dir_all(&config_root)?;
    symlink(config_root.join("missing.bones"), repo.join(".bones"))?;

    init_success(&env)?;

    let bones_dir = repo.join(".bones");
    assert!(bones_dir.join("bones.toml").is_file());
    let expected = env.home().join(".config/bonesdeploy/projects").join("atlas.bones");
    assert_eq!(fs::read_link(&bones_dir)?, expected);

    Ok(())
}
