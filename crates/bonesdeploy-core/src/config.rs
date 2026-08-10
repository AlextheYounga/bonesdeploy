use std::collections::BTreeMap;
use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::paths;

#[path = "app.rs"]
mod app;
pub use app::App;

#[path = "validation.rs"]
mod validation;
pub use validation::{is_numbered_shell_script, validate_project_name, validate_site_name};

/// Keys in the JSON object that bonesdeploy sends to bonesinfra.
pub mod bonesinfra_input {
    pub const SSH_PORT: &str = "ssh_port";
    pub const SSH_USER: &str = "ssh_user";
    pub const DEPLOY_USER: &str = "deploy_user";
    pub const PROJECT_NAME: &str = "project_name";
    pub const PROJECT_ROOT: &str = "project_root";
    pub const PREVIEW_DOMAIN: &str = "preview_domain";
    pub const REPO_PATH: &str = "repo_path";
    pub const RUNTIME_USER: &str = "runtime_user";
    pub const RUNTIME_GROUP: &str = "runtime_group";
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Bones {
    pub app: App,
    pub runtime: Runtime,
    pub services: Services,
    pub build: Build,
}

impl Deref for Bones {
    type Target = App;

    fn deref(&self) -> &Self::Target {
        &self.app
    }
}

impl DerefMut for Bones {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.app
    }
}

#[must_use]
pub fn default_deploy_user() -> String {
    paths::DEPLOY_USER.to_string()
}

/// # Errors
/// Returns an error when `port` is not a valid TCP port number.
pub fn parse_port(port: &str) -> Result<u16> {
    port.parse().with_context(|| format!("Invalid port: {port}"))
}

/// # Errors
/// Returns an error when `host` contains unsupported characters.
pub fn validate_host(host: &str) -> Result<()> {
    let host = host.trim();
    if host.is_empty() {
        return Ok(());
    }

    if host.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-')) {
        return Ok(());
    }

    bail!("Invalid host: {host}")
}

#[must_use]
pub fn runtime_user_for(project_name: &str) -> String {
    project_name.to_string()
}

#[must_use]
pub fn runtime_group_for(project_name: &str) -> String {
    project_name.to_string()
}

#[must_use]
pub fn build_user_for(project_name: &str) -> String {
    format!("{project_name}-build")
}

#[must_use]
pub fn build_group_for(project_name: &str) -> String {
    format!("{project_name}-build")
}

#[must_use]
pub fn default_repo_path_for(project_name: &str) -> String {
    paths::default_repo_path_for(project_name)
}

#[must_use]
pub fn default_preview_domain_for(project_name: &str, host: &str) -> String {
    let project = sanitize_domain_label(project_name);
    let host = sanitize_domain_label(host);

    if project.is_empty() || host.is_empty() {
        return String::new();
    }

    format!("{project}-{host}.nip.io")
}

fn sanitize_domain_label(value: &str) -> String {
    value
        .trim()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch.to_ascii_lowercase() } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Runtime {
    #[serde(default)]
    pub backend: RuntimeBackend,
    #[serde(default)]
    pub template: String,
    #[serde(default = "paths::default_web_root")]
    pub web_root: String,
    #[serde(default = "default_node_version")]
    pub node_version: String,
    #[serde(default)]
    pub shared: Shared,
    #[serde(default)]
    pub permissions: Option<toml::Value>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, toml::Value>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self {
            backend: RuntimeBackend::Native,
            template: String::new(),
            web_root: paths::default_web_root(),
            node_version: default_node_version(),
            shared: Shared::default(),
            permissions: None,
            extra: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeBackend {
    #[default]
    Native,
    Docker,
}

#[must_use]
pub fn default_node_version() -> String {
    String::from("24.18.0")
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Services {
    pub services: Vec<String>,
}

pub const BUILD_TIMEOUT_SECONDS_DEFAULT: u64 = 300;

/// Per-site build limits. Kept as its own nested section so build settings do
/// not leak into `BONES_*` environment variables as unrelated scalars.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Build {
    /// Maximum seconds each build script may run before systemd terminates it.
    /// `0` disables the timeout.
    #[serde(default = "default_build_timeout_seconds")]
    pub timeout_seconds: u64,
}

impl Default for Build {
    fn default() -> Self {
        Self { timeout_seconds: BUILD_TIMEOUT_SECONDS_DEFAULT }
    }
}

fn default_build_timeout_seconds() -> u64 {
    BUILD_TIMEOUT_SECONDS_DEFAULT
}

#[must_use]
pub fn build_timeout_seconds(config: &Bones) -> Option<u64> {
    (config.build.timeout_seconds != 0).then_some(config.build.timeout_seconds)
}

pub const DATABASE_SERVICES: &[&str] = &["postgres", "mariadb", "mysql", "mongodb", "valkey", "redis"];

/// # Errors
/// Returns an error when a configured database service is unsupported.
pub fn validate_database_services(services: &[String]) -> Result<()> {
    for service in services {
        if !DATABASE_SERVICES.contains(&service.as_str()) {
            bail!("unsupported database service: {service}");
        }
    }
    if services.iter().any(|service| service == "mariadb") && services.iter().any(|service| service == "mysql") {
        bail!("mariadb and mysql cannot be provisioned together; select one server implementation");
    }
    let mut unique = services.to_vec();
    unique.sort();
    unique.dedup();
    if unique.len() != services.len() {
        bail!("database services must not contain duplicates");
    }
    Ok(())
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Shared {
    pub paths: Vec<SharedPath>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedPath {
    pub path: String,
    #[serde(rename = "type")]
    pub path_type: SharedPathType,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SharedPathType {
    File,
    Dir,
}

/// # Errors
/// Returns an error when the configuration cannot be read or parsed.
pub fn load_runtime(config_dir: &Path) -> Result<Runtime> {
    let path = config_dir.join(paths::BONES_TOML);
    let content = fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))?;
    let bones: Bones = toml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))?;
    Ok(bones.runtime)
}

pub fn apply_derived_defaults(config: &mut Bones) {
    let project_name = config.project_name.clone();

    if config.ssh_user.is_empty() {
        config.ssh_user = String::from("root");
    }
    if config.preview_domain.is_empty() {
        config.preview_domain = default_preview_domain_for(&project_name, &config.host);
    }
}

/// Loads and parses a `bones.toml` configuration file, applying derived defaults.
/// # Errors
/// Returns an error if the file cannot be read or the TOML is invalid.
pub fn load(path: &Path) -> Result<Bones> {
    let content = fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let mut config: Bones = toml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))?;
    apply_derived_defaults(&mut config);
    validate_host(&config.host)?;
    validate_database_services(&config.services.services)?;
    Ok(config)
}
