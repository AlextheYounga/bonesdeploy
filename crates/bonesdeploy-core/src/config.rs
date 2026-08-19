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

pub mod project_env {
    pub const PROJECT_NAME: &str = "PROJECT_NAME";
    pub const REMOTE_NAME: &str = "REMOTE_NAME";
    pub const SSH_USER: &str = "SSH_USER";
    pub const HOST: &str = "HOST";
    pub const PORT: &str = "PORT";
    pub const BRANCH: &str = "BRANCH";
    pub const DOMAIN: &str = "DOMAIN";
    pub const PREVIEW_DOMAIN: &str = "PREVIEW_DOMAIN";
    pub const EMAIL: &str = "EMAIL";
    pub const SSL_ENABLED: &str = "SSL_ENABLED";
    pub const TEMPLATE: &str = "TEMPLATE";
    pub const RUNTIME_BACKEND: &str = "RUNTIME_BACKEND";
    pub const WEB_ROOT: &str = "WEB_ROOT";
    pub const SERVICES: &str = "SERVICES";
}

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
    String::from("24.18.0")
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Services {
    pub services: Vec<String>,
}

pub const BUILD_TIMEOUT_SECONDS_DEFAULT: u64 = 300;
pub const LARAVEL_INSTALL_QUEUE_WORKER: &str = "install_queue_worker";
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

/// Loads project-local provisioning values from a dotenv file.
///
/// The file is intentionally flat. Deployment paths and identities are derived
/// from the project name rather than persisted as a second configuration model.
/// # Errors
/// Returns an error if the file cannot be read or contains an invalid value.
pub fn load(path: &Path) -> Result<Bones> {
    let content = fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let values = parse_dotenv(&content)?;
    let project_name = values.get(project_env::PROJECT_NAME).cloned().unwrap_or_default();
    let mut config = Bones::default();
    config.project_name = project_name.clone();
    config.remote_name = values.get(project_env::REMOTE_NAME).cloned().unwrap_or_else(|| String::from("production"));
    config.ssh_user = values.get(project_env::SSH_USER).cloned().unwrap_or_else(|| String::from("root"));
    config.host = values.get(project_env::HOST).cloned().unwrap_or_default();
    config.port = values.get(project_env::PORT).cloned().unwrap_or_else(|| String::from("22"));
    config.branch = values.get(project_env::BRANCH).cloned().unwrap_or_else(|| String::from("main"));
    config.domain = values.get(project_env::DOMAIN).cloned().unwrap_or_default();
    config.preview_domain = values.get(project_env::PREVIEW_DOMAIN).cloned().unwrap_or_default();
    config.email = values.get(project_env::EMAIL).cloned().unwrap_or_default();
    config.ssl_enabled = values.get(project_env::SSL_ENABLED).is_some_and(|value| value == "true");
    config.runtime.template = values.get(project_env::TEMPLATE).cloned().unwrap_or_default();
    config.runtime.web_root = values.get(project_env::WEB_ROOT).cloned().unwrap_or_else(|| paths::default_web_root());
    config.runtime.backend = match values.get(project_env::RUNTIME_BACKEND).map_or("native", String::as_str) {
        "native" => RuntimeBackend::Native,
        "docker" => RuntimeBackend::Docker,
        value => bail!("Invalid {}: {value}", project_env::RUNTIME_BACKEND),
    };
    config.services.services = values
        .get(project_env::SERVICES)
        .map(|value| {
            value.split(',').filter(|item| !item.trim().is_empty()).map(|item| item.trim().to_string()).collect()
        })
        .unwrap_or_default();
    config.repo_path = default_repo_path_for(&project_name);
    config.project_root = paths::default_project_root_for(&project_name);
    config.preview_domain = if config.preview_domain.is_empty() {
        default_preview_domain_for(&project_name, &config.host)
    } else {
        config.preview_domain.clone()
    };
    validate_host(&config.host)?;
    validate_runtime(&config.runtime)?;
    validate_database_services(&config.services.services)?;
    Ok(config)
}

/// Validates flat dotenv content without constructing a configuration.
pub fn validate_dotenv(content: &str) -> Result<()> {
    parse_dotenv(content).map(|_| ())
}

/// Writes the flat project environment consumed by Rust, BonesInfra, and the
/// remote runtime loader.
///
/// # Errors
/// Returns an error when the environment file cannot be written.
pub fn save(config: &Bones, path: &Path) -> Result<()> {
    let runtime_backend = match config.runtime.backend {
        RuntimeBackend::Native => "native",
        RuntimeBackend::Docker => "docker",
    };
    let values = [
        (project_env::PROJECT_NAME, config.project_name.as_str()),
        (project_env::REMOTE_NAME, config.remote_name.as_str()),
        (project_env::HOST, config.host.as_str()),
        (project_env::PORT, config.port.as_str()),
        (project_env::SSH_USER, config.ssh_user.as_str()),
        (project_env::BRANCH, config.branch.as_str()),
        (project_env::DOMAIN, config.domain.as_str()),
        (project_env::PREVIEW_DOMAIN, config.preview_domain.as_str()),
        (project_env::EMAIL, config.email.as_str()),
        (project_env::SSL_ENABLED, if config.ssl_enabled { "true" } else { "false" }),
        (project_env::TEMPLATE, config.runtime.template.as_str()),
        (project_env::RUNTIME_BACKEND, runtime_backend),
        (project_env::WEB_ROOT, config.runtime.web_root.as_str()),
        (project_env::SERVICES, &config.services.services.join(",")),
    ];
    let mut content = String::new();
    for (key, value) in values {
        if value.contains('\n') || value.contains('\r') {
            bail!(".env values must not contain newlines");
        }
        content.push_str(&format!("{key}={}\n", format_dotenv_value(value)));
    }
    fs::write(path, content).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

fn parse_dotenv(content: &str) -> Result<BTreeMap<String, String>> {
    let mut values = BTreeMap::new();
    for (line_number, raw_line) in content.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            bail!("Invalid .env entry on line {}", line_number + 1);
        };
        let key = key.trim();
        if !is_valid_env_name(key) {
            bail!("Invalid .env key on line {}", line_number + 1);
        }
        if values.contains_key(key) {
            bail!("Duplicate .env key `{key}` on line {}", line_number + 1);
        }
        values.insert(key.to_string(), strip_quotes(value.trim()).to_string());
    }
    Ok(values)
}

fn is_valid_env_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else { return false };
    (first.is_ascii_alphabetic() || first == '_') && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn strip_quotes(value: &str) -> &str {
    let bytes = value.as_bytes();
    if bytes.len() >= 2
        && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\''))
    {
        &value[1..value.len() - 1]
    } else {
        value
    }
}

fn format_dotenv_value(value: &str) -> String {
    if value.trim() != value || value.starts_with(['"', '\'']) || value.ends_with(['"', '\'']) {
        format!("\"{value}\"")
    } else {
        value.to_string()
    }
}
