use anyhow::Result;
use bonesdeploy_core::config;
use bonesdeploy_core::config::{Bones, Build, BuildMode, Runtime, RuntimeBackend};
use bonesdeploy_core::config::{ProvisioningRequest, RemoteDeploymentConfig};
use std::fs;
use tempfile::tempdir;

#[test]
fn omitted_build_mode_defaults_to_remote_and_serializes_lowercase() -> Result<()> {
    let config: Bones = toml::from_str("[build]\ntimeout_seconds = 120\n")?;
    assert_eq!(config.build.mode, BuildMode::Remote);
    let json = serde_json::to_string(&config.build)?;
    assert!(json.contains("\"mode\":\"remote\""));
    assert_eq!(serde_json::from_str::<BuildMode>("\"local\"")?, BuildMode::Local);
    assert!(serde_json::from_str::<BuildMode>("\"LOCAL\"").is_err());
    Ok(())
}

#[test]
fn local_dotenv_build_mode_round_trips_and_is_managed() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join(".env");
    let mut bones = Bones::for_site("atlas");
    bones.build.mode = BuildMode::Local;

    config::write_local_environment(&bones, &path)?;
    let output = fs::read_to_string(&path)?;
    assert!(output.contains("BONES_BUILD_MODE=local\n"));
    assert_eq!(config::load(&path)?.build.mode, BuildMode::Local);

    fs::write(&path, "PROJECT_NAME=atlas\n")?;
    assert_eq!(config::load(&path)?.build.mode, BuildMode::Remote);
    Ok(())
}

#[test]
fn dotenv_build_mode_rejects_unknown_values() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join(".env");
    fs::write(&path, "PROJECT_NAME=atlas\nBUILD_MODE=LOCAL\n")?;
    let error = config::load(&path).expect_err("build mode must be lowercase");
    assert!(error.to_string().contains("Invalid BUILD_MODE"));
    Ok(())
}

#[test]
fn local_build_mode_rejects_docker_backend() {
    let runtime = Runtime { backend: RuntimeBackend::Docker, ..Runtime::default() };
    let build = Build { mode: BuildMode::Local, ..Build::default() };
    assert!(config::validate_build_mode(&runtime, &build).is_err());
}

#[test]
fn provisioning_uses_flat_build_mode_and_remote_config_keeps_timeout() -> Result<()> {
    let mut bones = Bones::for_site("atlas");
    bones.build.mode = BuildMode::Local;
    bones.build.timeout_seconds = 90;

    let request = ProvisioningRequest::from_bones(&bones)?;
    let site = request.site.as_ref().expect("site request carries fields");
    assert_eq!(site.build_mode, "local");
    let json = serde_json::to_string(&request)?;
    assert!(json.contains("\"build_mode\":\"local\""));
    assert!(!json.contains("\"timeout_seconds\""));
    assert!(!json.contains("\"build\""));

    let remote = RemoteDeploymentConfig::from_bones(&bones);
    let remote_json = serde_json::to_string(&remote)?;
    assert!(remote_json.contains("\"timeout_seconds\":90"));
    Ok(())
}

#[test]
fn provisioning_without_build_mode_defaults_to_remote() -> Result<()> {
    let json = r#"{"server":{"host":"h","ssh_user":"u","port":"22"},"site":{"project_name":"atlas","domain":"","email":"","ssl_enabled":false,"template":"","backend":"native","web_root":"public","branch":"main","node_version":"24.19.0","compose_port":null,"compose_wait_timeout":120,"backup":{"schedule":"0 0 * * *","retention_days":30,"passphrase":""},"extras":{}}}"#;
    let request: ProvisioningRequest = serde_json::from_str(json)?;
    assert_eq!(request.site.expect("site").build_mode, "remote");
    Ok(())
}
