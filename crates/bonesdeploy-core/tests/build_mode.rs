use anyhow::Result;
use bonesdeploy_core::config;
use bonesdeploy_core::config::Bones;
use bonesdeploy_core::config::{ProvisioningRequest, RemoteDeploymentConfig};
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
fn provisioning_and_remote_config_omit_build_settings() -> Result<()> {
    let mut bones = Bones::for_site("atlas");
    bones.build.timeout_seconds = 90;

    let request = ProvisioningRequest::from_bones(&bones)?;
    assert!(request.site.is_some());
    let json = serde_json::to_string(&request)?;
    assert!(!json.contains("\"build_mode\""));
    assert!(!json.contains("\"timeout_seconds\""));
    assert!(!json.contains("\"build\""));

    let remote = RemoteDeploymentConfig::from_bones(&bones);
    let remote_json = serde_json::to_string(&remote)?;
    assert!(!remote_json.contains("\"build\""));
    Ok(())
}

#[test]
fn provisioning_rejects_removed_build_mode() {
    let json = r#"{"server":{"host":"h","ssh_user":"u","port":"22"},"site":{"project_name":"atlas","domain":"","email":"","ssl_enabled":false,"template":"","backend":"native","web_root":"public","branch":"main","node_version":"24.19.0","compose_port":null,"compose_wait_timeout":120,"build_mode":"remote","backup":{"schedule":"0 0 * * *","retention_days":30,"passphrase":""},"extras":{}}}"#;
    assert!(serde_json::from_str::<ProvisioningRequest>(json).is_err());
}
