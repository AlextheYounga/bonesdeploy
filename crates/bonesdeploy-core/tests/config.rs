//! Public config parsing and defaults of the `bonesdeploy-core` library.

use anyhow::Result;
use bonesdeploy_core::config;
use bonesdeploy_core::config::BUILD_TIMEOUT_SECONDS_DEFAULT;
use bonesdeploy_core::config::{
    App, Bones, Build, Runtime, RuntimeBackend, build_timeout_seconds, validate_host, validate_runtime,
};
use std::collections::BTreeMap;
use std::fs;
use tempfile::tempdir;
use toml::de::Error;
use toml::map::Map;

#[test]
fn omitted_nested_sections_keep_app_defaults() -> Result<(), Error> {
    let app: App = toml::from_str("")?;

    assert_eq!(app.ssh_user, "root");
    assert_eq!(app.port, "22");
    assert_eq!(app.branch, "master");
    assert_eq!(app.releases_keep, 5);
    Ok(())
}

#[test]
fn validate_host_accepts_hostnames_and_ips() {
    assert!(validate_host("deploy.example.com").is_ok());
    assert!(validate_host("192.0.2.10").is_ok());
    assert!(validate_host("").is_ok());
}

#[test]
fn validate_host_rejects_shell_metacharacters() {
    assert!(validate_host("deploy.example.com;rm -rf /").is_err());
}

#[test]
fn runtime_backend_defaults_to_native() -> Result<(), Error> {
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
fn removed_runtime_shared_configuration_is_rejected() {
    let runtime = Runtime {
        extra: BTreeMap::from([(String::from("shared"), toml::Value::Table(Map::new()))]),
        ..Runtime::default()
    };

    assert!(validate_runtime(&runtime).is_err());
}

#[test]
fn build_timeout_defaults_to_five_minutes() {
    let config = Bones::default();
    assert_eq!(config.build.timeout_seconds, BUILD_TIMEOUT_SECONDS_DEFAULT);
    assert_eq!(build_timeout_seconds(&config), Some(BUILD_TIMEOUT_SECONDS_DEFAULT));
}

#[test]
fn build_timeout_of_zero_disables_the_timeout() {
    let config = Bones { build: Build { timeout_seconds: 0 }, ..Bones::default() };
    assert_eq!(build_timeout_seconds(&config), None);
}

#[test]
fn build_timeout_parses_from_toml() -> Result<()> {
    let config: Bones = toml::from_str("[build]\ntimeout_seconds = 120\n")?;
    assert_eq!(build_timeout_seconds(&config), Some(120));
    Ok(())
}

#[test]
fn dotenv_rejects_invalid_and_duplicate_keys() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join(".env");
    fs::write(&path, "PROJECT_NAME=atlas\nBAD-KEY=value\n")?;
    assert!(config::load(&path).is_err());
    fs::write(&path, "PROJECT_NAME=atlas\nPROJECT_NAME=other\n")?;
    assert!(config::load(&path).is_err());
    Ok(())
}

#[test]
fn dotenv_merge_preserves_existing_values_and_replaces_overlaid_keys() -> Result<()> {
    let existing = "PROJECT_NAME=old\nDATABASE_PASSWORD=generated\nAPP_KEY=old\n";
    let secrets = "APP_KEY=secret\nNODE_ENV=production\n";
    let project = "PROJECT_NAME=e2evue\nHOST=192.0.2.1\n";

    let merged = config::merge_dotenv(existing, secrets)?;
    let merged = config::merge_dotenv(&merged, project)?;

    assert!(merged.contains("DATABASE_PASSWORD=generated\n"));
    assert!(merged.contains("APP_KEY=secret\n"));
    assert!(merged.contains("PROJECT_NAME=e2evue\n"));
    assert!(!merged.contains("PROJECT_NAME=old\n"));
    assert!(!merged.contains("APP_KEY=old\n"));
    Ok(())
}
