mod common;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use common::TestEnv;

const INIT_ARGS: &[&str] = &["init", "--non-interactive", "--project-name", "atlas", "--host", "deploy.example.com"];

fn init_success(env: &TestEnv) -> Result<()> {
    let output = env.run(INIT_ARGS)?;
    assert!(output.status.success(), "init failed: {}", String::from_utf8_lossy(&output.stderr));
    Ok(())
}

fn assert_project_infra(repo: &Path) -> Result<()> {
    let infra = repo.join("infra");
    assert!(infra.join("provision/core/pyproject.toml").is_file());
    assert!(infra.join("provision/core/uv.lock").is_file());
    assert!(infra.join("provision/core/src/bonesinfra/__main__.py").is_file());
    assert!(infra.join("provision/core/src/bonesinfra/frameworks/custom/runtime.py").is_file());
    assert!(infra.join("provision/custom/__init__.py").is_file());
    assert!(infra.join("provision/custom/runtime.py").is_file());
    assert!(infra.join("provision/custom/manifest.py").is_file());
    assert!(infra.join("secrets").is_dir());
    let deploy_dir = infra.join("deployment");
    assert!(deploy_dir.is_dir());
    assert!(deploy_dir.read_dir()?.next().is_some(), "deployment directory should have scripts");
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

    assert_project_infra(&repo)?;
    assert!(!repo.join(".bones").exists());
    assert!(!repo.join("bones.toml").exists());

    let env_build = repo.join(".env.build");
    assert!(env_build.is_file(), ".env.build should be created");
    let env_build_content = fs::read_to_string(&env_build)?;
    assert!(env_build_content.contains("Do not place passwords"));

    let gitignore = fs::read_to_string(repo.join(".gitignore"))?;
    assert!(gitignore.lines().any(|line| line.trim() == "!.env.build"));

    let gitignore = fs::read_to_string(repo.join(".gitignore"))?;
    assert!(gitignore.lines().any(|line| line.trim() == ".env"));
    assert!(!atlas_config_root(&env).exists());

    Ok(())
}

#[test]
fn named_framework_materializes_project_infrastructure_snapshot() -> Result<()> {
    let framework = "laravel";
    let env = TestEnv::new()?;
    let output = env.run(&[
        "init",
        "--non-interactive",
        "--project-name",
        "atlas",
        "--host",
        "deploy.example.com",
        "--template",
        framework,
    ])?;
    assert!(output.status.success(), "{framework} init failed: {}", String::from_utf8_lossy(&output.stderr));
    let infra = env.repo().join("infra");
    assert!(
        infra.join("provision/core/src/bonesinfra/frameworks/laravel/runtime.py").is_file(),
        "{framework} is missing core runtime"
    );
    assert!(infra.join("provision/custom/runtime.py").is_file(), "{framework} is missing custom runtime");
    assert!(infra.join("deployment/functions.sh").is_file(), "{framework} is missing kit deployment functions");
    assert!(
        infra.join("provision/core/src/bonesinfra/frameworks/laravel/templates").is_dir(),
        "{framework} is missing infra templates"
    );
    assert!(!env.repo().join(".bones").exists());
    Ok(())
}

#[test]
fn rerun_preserves_existing_bones_assets() -> Result<()> {
    let env = TestEnv::new()?;
    let sentinel = env.repo().join("infra/provision/core/src/bonesinfra/project.py");

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
fn rejects_old_bones_layout() -> Result<()> {
    let env = TestEnv::new()?;
    let repo = env.repo();
    fs::create_dir(&repo.join(".bones"))?;

    let output = env.run(INIT_ARGS)?;
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("bonesdeploy update"));

    Ok(())
}
