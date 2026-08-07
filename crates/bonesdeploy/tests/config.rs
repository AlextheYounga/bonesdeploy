mod common;

use std::fs;
use std::path::PathBuf;

use anyhow::{Result, bail};
use common::TestEnv;

fn write_config(env: &TestEnv, toml: &str) -> Result<PathBuf> {
    let path = env.repo().join("bones.toml");
    fs::write(&path, toml)?;
    Ok(path)
}

fn read_value(env: &TestEnv, path: &PathBuf, key: Option<&str>) -> Result<String> {
    let file = path.to_str().ok_or_else(|| anyhow::anyhow!("non-UTF8 temp path"))?;
    let mut args = vec!["config", "--file", file];
    if let Some(key) = key {
        args.push(key);
    }
    let output = env.run(&args)?;
    if !output.status.success() {
        bail!("config failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[test]
fn dumps_full_file_when_key_omitted() -> Result<()> {
    let env = TestEnv::new()?;
    let toml = "[app]\nproject_name = \"atlas\"\n";
    let path = write_config(&env, toml)?;
    assert_eq!(read_value(&env, &path, None)?, toml);
    Ok(())
}

#[test]
fn reads_string_key() -> Result<()> {
    let env = TestEnv::new()?;
    let path = write_config(&env, "[app]\nproject_name = \"atlas\"\n")?;
    assert_eq!(read_value(&env, &path, Some("app.project_name"))?, "atlas");
    Ok(())
}

#[test]
fn reads_integer_key() -> Result<()> {
    let env = TestEnv::new()?;
    let path = write_config(&env, "[app.deploy]\nreleases = 5\n")?;
    assert_eq!(read_value(&env, &path, Some("app.deploy.releases"))?, "5");
    Ok(())
}

#[test]
fn missing_key_bails() -> Result<()> {
    let env = TestEnv::new()?;
    let path = write_config(&env, "[app]\nproject_name = \"atlas\"\n")?;
    let file = path.to_str().ok_or_else(|| anyhow::anyhow!("non-UTF8 temp path"))?;
    let output = env.run(&["config", "--file", file, "nope"])?;

    assert!(!output.status.success(), "missing key should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found"), "missing key error should explain: {stderr}");
    Ok(())
}
