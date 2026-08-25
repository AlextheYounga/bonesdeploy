//! Public config parsing and defaults of the `bonesdeploy-core` library.

use anyhow::Result;
use bonesdeploy_core::config;
use bonesdeploy_core::config::BUILD_TIMEOUT_SECONDS_DEFAULT;
use bonesdeploy_core::config::{
    App, Bones, Build, RemoteDeploymentConfig, Runtime, RuntimeBackend, build_timeout_seconds, validate_host,
    validate_runtime,
};
use bonesdeploy_core::paths;
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

#[test]
fn dotenv_round_trips_framework_values() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join(".env");
    let mut config = Bones::default();
    config.runtime.extra.insert(String::from("is_static"), toml::Value::Boolean(true));

    config::save(&config, &path)?;
    assert!(fs::read_to_string(&path)?.contains("IS_STATIC=true\n"));
    let loaded = config::load(&path)?;

    assert_eq!(loaded.runtime.extra.get("is_static"), Some(&toml::Value::Boolean(true)));
    Ok(())
}

#[test]
fn remote_deployment_config_excludes_identity_and_secrets() -> Result<()> {
    let mut bones = Bones::for_site("mysite");
    bones.branch = "main".to_string();
    bones.releases_keep = 3;
    bones.runtime.backend = RuntimeBackend::Docker;
    bones.runtime.web_root = "public".to_string();
    bones.build.timeout_seconds = 120;
    bones.host = "example.com".to_string();
    bones.ssh_user = "root".to_string();
    bones.port = "2222".to_string();
    bones.domain = "myapp.com".to_string();
    bones.ssl_enabled = true;

    let descriptor = RemoteDeploymentConfig::from_bones(&bones);

    let json = serde_json::to_string(&descriptor)?;
    assert!(!json.contains("project_name"));
    assert!(!json.contains("host"));
    assert!(!json.contains("ssh_user"));
    assert!(!json.contains("port"));
    assert!(!json.contains("domain"));
    assert!(!json.contains("ssl_enabled"));
    assert!(!json.contains("repo_path"));
    assert!(!json.contains("project_root"));
    assert!(!json.contains("remote_name"));
    assert!(json.contains("\"branch\":\"main\""));
    assert!(json.contains("\"releases_keep\":3"));
    assert!(json.contains("\"backend\":\"docker\""));
    assert!(json.contains("\"web_root\":\"public\""));
    assert!(json.contains("\"timeout_seconds\":120"));
    Ok(())
}

#[test]
fn remote_deployment_config_round_trips_through_json() -> Result<()> {
    let mut bones = Bones::for_site("atlas");
    bones.branch = "develop".to_string();
    bones.releases_keep = 7;
    bones.runtime.web_root = "dist".to_string();
    bones.runtime.backend = RuntimeBackend::Native;
    bones.build.timeout_seconds = 600;

    let descriptor = RemoteDeploymentConfig::from_bones(&bones);
    let json = serde_json::to_string(&descriptor)?;
    let restored: RemoteDeploymentConfig = serde_json::from_str(&json)?;

    assert_eq!(restored.branch, "develop");
    assert_eq!(restored.releases_keep, 7);
    assert_eq!(restored.runtime.web_root, "dist");
    assert_eq!(restored.runtime.backend, RuntimeBackend::Native);
    assert_eq!(restored.build.timeout_seconds, 600);
    Ok(())
}

#[test]
fn remote_deployment_config_into_site_config_derives_identity_from_site() {
    let mut bones = Bones::for_site("original");
    bones.branch = "release".to_string();
    bones.releases_keep = 2;
    bones.runtime.web_root = "build".to_string();

    let descriptor = RemoteDeploymentConfig::from_bones(&bones);
    let site_config = descriptor.into_site_config("target-site");

    assert_eq!(site_config.project_name, "target-site");
    assert_eq!(site_config.project_root, paths::default_project_root_for("target-site"));
    assert_eq!(site_config.repo_path, paths::default_repo_path_for("target-site"));
    assert_eq!(site_config.branch, "release");
    assert_eq!(site_config.releases_keep, 2);
    assert_eq!(site_config.runtime.web_root, "build");
    // Host and SSH settings are not carried by the descriptor.
    assert!(site_config.host.is_empty());
    assert_eq!(site_config.ssh_user, "root");
}

#[test]
fn remote_deployment_config_rejects_unknown_fields() {
    let json = r#"{"branch":"main","releases_keep":5,"runtime":{"backend":"native","template":"","web_root":"public","node_version":"24.19.0"},"build":{"timeout_seconds":300},"extra_field":"bad"}"#;
    let result: Result<RemoteDeploymentConfig, _> = serde_json::from_str(json);
    assert!(result.is_err());
}
