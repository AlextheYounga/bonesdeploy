use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::paths;

#[path = "app.rs"]
mod app;
pub use app::App;

mod dotenv;
pub use dotenv::{load, merge_dotenv, project_env, save, validate_dotenv};

pub mod environment;

#[path = "validation.rs"]
mod validation;
pub use validation::{is_numbered_shell_script, validate_project_name, validate_site_name};

pub const PROJECT_SETUP_ERROR: &str = "root .env and infra/ are required. Run `bonesdeploy init` first.";

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Bones {
    pub app: App,
    pub runtime: Runtime,
    pub services: Services,
    pub build: Build,
}

impl Bones {
    /// Builds the remote-side identity and convention-derived paths for a site.
    /// Runtime settings remain at their defaults until a lifecycle operation
    /// loads the deployed shared environment.
    #[must_use]
    pub fn for_site(site: &str) -> Self {
        let mut config = Self::default();
        config.project_name = site.to_string();
        config.repo_path = default_repo_path_for(site);
        config.project_root = paths::default_project_root_for(site);
        config
    }
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
    String::from("24.19.0")
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Services {
    pub services: Vec<String>,
}

pub const BUILD_TIMEOUT_SECONDS_DEFAULT: u64 = 300;
pub const LARAVEL_TEMPLATE: &str = "laravel";

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

/// Reject the removed shared-path configuration instead of silently ignoring it.
///
/// Frameworks own their shared directory declarations now; the only managed
/// shared file is always `shared/.env`.
pub fn validate_runtime(runtime: &Runtime) -> Result<()> {
    if runtime.extra.contains_key("shared") {
        bail!(
            "runtime.shared is no longer supported; shared/.env and framework directories are provisioned automatically"
        );
    }
    Ok(())
}

/// # Errors
/// Returns an error when the local environment file cannot be read or parsed.
pub fn load_runtime(config_dir: &Path) -> Result<Runtime> {
    let path = config_dir.join(paths::DOT_ENV);
    Ok(load(&path)?.runtime)
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
