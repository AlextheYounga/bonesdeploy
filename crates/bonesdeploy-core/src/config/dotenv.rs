use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};

use super::{Bones, RuntimeBackend, default_preview_domain_for, default_repo_path_for, validate_database_services};
use super::{validate_host, validate_runtime};
use crate::paths;

pub mod project_env {
    pub use super::super::environment::{PROJECT_NAME, WEB_ROOT};

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
    pub const SERVICES: &str = "SERVICES";
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
    for (key, value) in &values {
        if !is_project_env_key(key) {
            config.runtime.extra.insert(key.to_ascii_lowercase(), parse_runtime_value(value));
        }
    }
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

/// Merges dotenv content while preserving the original text of retained values.
/// Values from `overlay` replace values with the same key in `base`.
///
/// # Errors
/// Returns an error when either layer is not valid dotenv content.
pub fn merge_dotenv(base: &str, overlay: &str) -> Result<String> {
    validate_dotenv(base)?;
    let overlay_values = parse_dotenv(overlay)?;
    let mut merged = String::new();
    for line in base.lines() {
        let key = line.trim().split_once('=').map(|(key, _)| key.trim());
        if key.is_some_and(|key| overlay_values.contains_key(key)) {
            continue;
        }
        merged.push_str(line);
        merged.push('\n');
    }
    if !overlay.is_empty() {
        merged.push_str(overlay);
        if !overlay.ends_with('\n') {
            merged.push('\n');
        }
    }
    Ok(merged)
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
    for (key, value) in &config.runtime.extra {
        let value = runtime_value_to_string(value)
            .with_context(|| format!("Runtime framework value `{key}` must be a scalar"))?;
        if value.contains('\n') || value.contains('\r') {
            bail!(".env values must not contain newlines");
        }
        content.push_str(&format!("{}={}\n", key.to_ascii_uppercase(), format_dotenv_value(&value)));
    }
    fs::write(path, content).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

fn is_project_env_key(key: &str) -> bool {
    matches!(
        key,
        project_env::PROJECT_NAME
            | project_env::REMOTE_NAME
            | project_env::SSH_USER
            | project_env::HOST
            | project_env::PORT
            | project_env::BRANCH
            | project_env::DOMAIN
            | project_env::PREVIEW_DOMAIN
            | project_env::EMAIL
            | project_env::SSL_ENABLED
            | project_env::TEMPLATE
            | project_env::RUNTIME_BACKEND
            | project_env::WEB_ROOT
            | project_env::SERVICES
    )
}

fn parse_runtime_value(value: &str) -> toml::Value {
    match value {
        "true" => toml::Value::Boolean(true),
        "false" => toml::Value::Boolean(false),
        _ => toml::Value::String(value.to_string()),
    }
}

fn runtime_value_to_string(value: &toml::Value) -> Option<String> {
    match value {
        toml::Value::String(value) => Some(value.clone()),
        toml::Value::Integer(value) => Some(value.to_string()),
        toml::Value::Float(value) => Some(value.to_string()),
        toml::Value::Boolean(value) => Some(value.to_string()),
        toml::Value::Datetime(value) => Some(value.to_string()),
        toml::Value::Array(_) | toml::Value::Table(_) => None,
    }
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
