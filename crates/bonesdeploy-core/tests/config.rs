//! Public config parsing and defaults of the `bonesdeploy-core` library.

use anyhow::Result;
use bonesdeploy_core::config::BUILD_TIMEOUT_SECONDS_DEFAULT;
use bonesdeploy_core::config::{App, Bones, Build, Runtime, SharedPathType, build_timeout_seconds, validate_host};
use toml::de::Error;

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
fn runtime_parses_shared_paths() -> Result<()> {
    let runtime: Runtime = toml::from_str(
        r#"
web_root = "public"

[shared]
paths = [
    { path = ".env", type = "file" },
    { path = "storage", type = "dir" },
]
"#,
    )?;

    assert_eq!(runtime.shared.paths.len(), 2);
    assert_eq!(runtime.shared.paths[0].path, ".env");
    assert_eq!(runtime.shared.paths[0].path_type, SharedPathType::File);
    assert_eq!(runtime.shared.paths[1].path, "storage");
    assert_eq!(runtime.shared.paths[1].path_type, SharedPathType::Dir);
    Ok(())
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
