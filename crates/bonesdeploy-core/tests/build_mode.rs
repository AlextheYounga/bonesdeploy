use anyhow::Result;
use bonesdeploy_core::config;
use bonesdeploy_core::config::Bones;
use bonesdeploy_core::config::RemoteDeploymentConfig;
use std::fs;
use tempfile::tempdir;

#[test]
fn build_timeout_is_local_configuration_without_a_mode() -> Result<()> {
    let config: Bones = toml::from_str("[build]\ntimeout_seconds = 120\n")?;
    assert_eq!(config.build.timeout_seconds, 120);
    let json = serde_json::to_string(&config.build)?;
    assert_eq!(json, r#"{"timeout_seconds":120}"#);
    Ok(())
}

#[test]
fn dotenv_rejects_removed_build_mode() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join(".env");
    fs::write(&path, "BONES_BUILD_MODE=local\n")?;
    let error = config::load(&path).expect_err("removed build mode must fail");
    assert!(error.to_string().contains("no longer supported"));
    Ok(())
}

#[test]
fn remote_config_omits_local_build_settings() -> Result<()> {
    let mut bones = Bones::for_site("atlas");
    bones.build.timeout_seconds = 90;

    let remote = RemoteDeploymentConfig::from_bones(&bones);
    let remote_json = serde_json::to_string(&remote)?;
    assert!(!remote_json.contains("\"branch\""));
    assert!(!remote_json.contains("\"build\""));
    Ok(())
}
