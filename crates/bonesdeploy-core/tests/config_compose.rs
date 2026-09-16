use anyhow::Result;
use bonesdeploy_core::config::{self, Bones, COMPOSE_WAIT_TIMEOUT_DEFAULT, Runtime, RuntimeBackend, validate_runtime};
use std::fs;
use tempfile::tempdir;

#[test]
fn runtime_backend_defaults_to_native() -> Result<()> {
    let runtime: Runtime = toml::from_str("")?;

    assert_eq!(runtime.backend, RuntimeBackend::Native);
    Ok(())
}

#[test]
fn runtime_backend_serializes_as_lowercase_toml() -> Result<()> {
    let runtime = Runtime { backend: RuntimeBackend::Docker, ..Runtime::default() };

    let value = toml::to_string(&runtime)?;

    assert!(value.lines().any(|line| line == "backend = \"docker\""));
    Ok(())
}

#[test]
fn compose_runtime_configuration_defaults_and_validates() -> Result<()> {
    let runtime: Runtime = toml::from_str("")?;
    assert_eq!(runtime.compose_port, None);
    assert_eq!(runtime.compose_wait_timeout, COMPOSE_WAIT_TIMEOUT_DEFAULT);
    assert!(
        validate_runtime(&Runtime { compose_port: Some(1), compose_wait_timeout: 1, ..Runtime::default() }).is_ok()
    );
    assert!(validate_runtime(&Runtime { compose_port: Some(0), ..Runtime::default() }).is_err());
    assert!(validate_runtime(&Runtime { compose_wait_timeout: 0, ..Runtime::default() }).is_err());
    assert!(validate_runtime(&Runtime { compose_wait_timeout: 3601, ..Runtime::default() }).is_err());
    Ok(())
}

#[test]
fn dotenv_round_trips_compose_runtime_configuration() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join(".env");
    let mut bones = Bones::default();
    bones.runtime.compose_port = Some(8080);
    bones.runtime.compose_wait_timeout = 240;

    config::write_local_environment(&bones, &path)?;
    let output = fs::read_to_string(&path)?;
    assert!(output.contains("BONES_COMPOSE_PORT=8080"));
    assert!(output.contains("BONES_COMPOSE_WAIT_TIMEOUT=240"));
    let loaded = config::load(&path)?;
    assert_eq!(loaded.runtime.compose_port, Some(8080));
    assert_eq!(loaded.runtime.compose_wait_timeout, 240);
    Ok(())
}
