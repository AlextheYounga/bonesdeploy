//! Public config parsing and defaults of the `bonesdeploy-core` library.

use anyhow::Result;
use bonesdeploy_core::config;
use bonesdeploy_core::config::BUILD_TIMEOUT_SECONDS_DEFAULT;
use bonesdeploy_core::config::{
    App, Bones, Build, ParsedDotEnv, ProvisioningRequest, Runtime, build_timeout_seconds, default_deploy_user,
    parse_port, production_application_keys, validate_host, validate_runtime,
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
    assert_eq!(app.branch, "main");
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
fn parse_port_rejects_zero_and_values_outside_tcp_range() {
    assert_eq!(parse_port("22").ok(), Some(22));
    assert!(parse_port("0").is_err());
    assert!(parse_port("65536").is_err());
    assert!(parse_port("not-a-port").is_err());
}

#[test]
fn deploy_user_defaults_to_deploy() {
    assert_eq!(default_deploy_user(), "deploy");
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
fn dotenv_round_trips_framework_values() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join(".env");
    let mut config = Bones::default();
    config.runtime.extra.insert(String::from("is_static"), toml::Value::Boolean(true));

    config::write_local_environment(&config, &path)?;
    assert!(fs::read_to_string(&path)?.contains("BONES_IS_STATIC=true\n"));
    let loaded = config::load(&path)?;

    assert_eq!(loaded.runtime.extra.get("is_static"), Some(&toml::Value::Boolean(true)));
    Ok(())
}

#[test]
fn managed_block_delimiters_replaced_atomically_preserving_application_content() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join(".env");
    fs::write(
        &path,
        "# app\nAPP_TOKEN=\"quoted value\"\n\n# >>> BonesDeploy managed configuration >>>\nPROJECT_NAME=old\n# ignored\n# <<< BonesDeploy managed configuration <<<\n",
    )?;
    let mut bones = Bones::default();
    bones.project_name = "new".into();
    config::write_local_environment(&bones, &path)?;
    let output = fs::read_to_string(&path)?;
    assert!(output.starts_with("# app\nAPP_TOKEN=\"quoted value\"\n\n"));
    assert_eq!(output.matches("# >>> BonesDeploy managed configuration >>>").count(), 1);
    assert_eq!(output.matches("BONES_PROJECT_NAME=new\n").count(), 1);
    assert!(!output.contains("PROJECT_NAME=old"));
    assert!(output.ends_with('\n'));
    Ok(())
}

#[test]
fn flat_configuration_absorbed_into_managed_block_on_load() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join(".env");
    fs::write(
        &path,
        "PROJECT_NAME=atlas\nHOST=192.0.2.1\nSSL_ENABLED=true\nCOMPOSE_PORT=8080\nCOMPOSE_WAIT_TIMEOUT=240\nIS_STATIC=true\n",
    )?;
    let loaded = config::load_local(&path)?;
    assert_eq!(loaded.environment.project_name, "atlas");
    assert_eq!(loaded.environment.runtime.compose_port, Some(8080));
    assert_eq!(loaded.environment.runtime.compose_wait_timeout, 240);
    assert!(loaded.environment.runtime.extra.contains_key("is_static"));
    let parsed = config::parse_dotenv(&fs::read_to_string(&path)?)?;
    assert!(parsed.needs_rewrite);
    config::write_local_environment(&loaded.environment, &path)?;
    let output = fs::read_to_string(&path)?;
    assert!(output.contains("BONES_IS_STATIC=true\n"));
    assert!(!output.lines().any(|line| line == "PROJECT_NAME=atlas"));
    Ok(())
}

#[test]
fn dotenv_rejects_removed_built_in_service_configuration() {
    let content = "# >>> BonesDeploy managed configuration >>>\nBONES_SERVICES=postgres\n# <<< BonesDeploy managed configuration <<<\n";
    assert!(config::validate_dotenv(content).is_err());
}

#[test]
fn dotenv_rejects_removed_deployment_remote_configuration() {
    let content = "# >>> BonesDeploy managed configuration >>>\nBONES_REMOTE_NAME=production\n# <<< BonesDeploy managed configuration <<<\n";
    assert!(config::validate_dotenv(content).is_err());
    assert!(config::validate_dotenv("REMOTE_NAME=production\n").is_err());
}

#[test]
fn reserved_bones_prefix_outside_block_is_rejected() {
    let error = config::validate_dotenv("BONES_UNKNOWN=value\n").expect_err("reserved key");
    assert!(error.to_string().contains("BONES_UNKNOWN"));
    assert!(error.to_string().contains("managed"));
}

#[test]
fn duplicates_across_flat_and_block_forms_are_rejected() {
    let content = "PROJECT_NAME=one\n# >>> BonesDeploy managed configuration >>>\nBONES_PROJECT_NAME=two\n# <<< BonesDeploy managed configuration <<<\n";
    assert!(config::validate_dotenv(content).is_err());
}

#[test]
fn production_filter_excludes_every_managed_source_and_retains_applications() -> Result<()> {
    let parsed = ParsedDotEnv {
        managed: BTreeMap::from([(String::from("PROJECT_NAME"), String::from("atlas"))]),
        applications: BTreeMap::from([
            (String::from("PROJECT_NAME"), String::from("flat")),
            (String::from("API_TOKEN"), String::from("secret")),
            (String::from("APP_VALUE_2"), String::from("two")),
        ]),
        needs_rewrite: false,
    };
    let applications = production_application_keys(&parsed)?;
    assert_eq!(
        applications,
        BTreeMap::from([
            (String::from("API_TOKEN"), String::from("secret")),
            (String::from("APP_VALUE_2"), String::from("two"))
        ])
    );
    Ok(())
}

#[test]
fn provisioning_request_round_trips_through_json() -> Result<()> {
    let mut bones = Bones::default();
    bones.host = "example.com".into();
    bones.runtime.extra.insert("php_version".into(), toml::Value::Boolean(true));
    let request = ProvisioningRequest::from_bones(&bones)?;
    let json = serde_json::to_string(&request)?;
    let restored: ProvisioningRequest = serde_json::from_str(&json)?;
    assert_eq!(request, restored);
    assert!(serde_json::to_string(&ProvisioningRequest::server_only("h", "u", "22"))?.find("site").is_none());
    assert!(
        serde_json::from_str::<ProvisioningRequest>(
            r#"{"server":{"host":"h","ssh_user":"u","port":"22","intruder":1}}"#
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn dotenv_round_trips_backup_configuration() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join(".env");
    let mut config = Bones::default();
    config.project_name = "atlas".into();
    config.backup.schedule = "30 3 * * 0".into();
    config.backup.retention_days = 14;
    config.backup.passphrase = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".into();

    config::write_local_environment(&config, &path)?;
    let output = fs::read_to_string(&path)?;
    assert!(output.contains("BONES_BORG_PASSPHRASE=0123456789abcdef"));
    assert!(output.contains("BONES_BACKUP_SCHEDULE=30 3 * * 0"));
    assert!(output.contains("BONES_BACKUP_RETENTION_DAYS=14"));
    let loaded = config::load(&path)?;

    assert_eq!(loaded.backup.schedule, "30 3 * * 0");
    assert_eq!(loaded.backup.retention_days, 14);
    assert_eq!(loaded.backup.passphrase, "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");
    assert!(loaded.backup.is_configured());
    Ok(())
}

#[test]
fn environment_without_backup_keys_loads_with_backup_defaults() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join(".env");
    fs::write(&path, "PROJECT_NAME=atlas\n")?;

    let loaded = config::load(&path)?;

    assert_eq!((loaded.backup.schedule.as_str(), loaded.backup.retention_days), ("0 0 * * *", 30));
    assert!(loaded.backup.passphrase.is_empty());
    assert!(!loaded.backup.is_configured());
    Ok(())
}

#[test]
fn provisioning_request_carries_backup_fields_including_the_passphrase() -> Result<()> {
    let mut bones = Bones::for_site("atlas");
    bones.backup.schedule = "15 2 * * *".into();
    bones.backup.retention_days = 21;
    bones.backup.passphrase = "hex-passphrase".into();

    let request = ProvisioningRequest::from_bones(&bones)?;
    let json = serde_json::to_string(&request)?;

    let site = request.site.expect("site request carries backup fields");
    assert_eq!(
        (site.backup.schedule.as_str(), site.backup.retention_days, site.backup.passphrase.as_str()),
        ("15 2 * * *", 21, "hex-passphrase")
    );
    assert!(json.contains("passphrase"));
    Ok(())
}

#[test]
fn header_synthesis_when_file_absent_creates_normalized_environment() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join(".env");
    let bones = Bones::default();
    config::write_local_environment(&bones, &path)?;
    let output = fs::read_to_string(&path)?;
    assert!(output.starts_with("# Local environment for the application.\n\n"));
    assert!(output.contains("# >>> BonesDeploy managed configuration >>>\n"));
    assert_eq!(config::load(&path)?.branch, bones.branch);
    Ok(())
}
