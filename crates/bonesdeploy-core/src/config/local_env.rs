use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use anyhow::{Context, Result, bail};

use super::atomic_write::atomic_write;
use super::backup::validate_backup;
use super::model::{
    Bones, RuntimeBackend, default_node_version, default_repo_path_for, validate_database_services, validate_host,
    validate_runtime,
};
use crate::paths;

mod keys {
    pub(super) use super::super::backup::{PASSPHRASE_KEY, RETENTION_DAYS_KEY, SCHEDULE_KEY};
    /// Managed-key vocabulary for the root `.env` grammar.
    pub(super) use crate::config::variables::{PROJECT_NAME, WEB_ROOT};
    pub(super) const REMOTE_NAME: &str = "REMOTE_NAME";
    pub(super) const SSH_USER: &str = "SSH_USER";
    pub(super) const HOST: &str = "HOST";
    pub(super) const PORT: &str = "PORT";
    pub(super) const BRANCH: &str = "BRANCH";
    pub(super) const DOMAIN: &str = "DOMAIN";
    pub(super) const EMAIL: &str = "EMAIL";
    pub(super) const SSL_ENABLED: &str = "SSL_ENABLED";
    pub(super) const TEMPLATE: &str = "TEMPLATE";
    pub(super) const RUNTIME_BACKEND: &str = "RUNTIME_BACKEND";
    pub(super) const SERVICES: &str = "SERVICES";
    pub(super) const NODE_VERSION: &str = "NODE_VERSION";
}

const BEGIN: &str = "# >>> BonesDeploy managed configuration >>>";
const END: &str = "# <<< BonesDeploy managed configuration <<<";
pub(crate) const MANAGED_PREFIX: &str = "BONES_";
const MANAGED: &[&str] = &[
    keys::PROJECT_NAME,
    keys::REMOTE_NAME,
    keys::SSH_USER,
    keys::HOST,
    keys::PORT,
    keys::BRANCH,
    keys::DOMAIN,
    keys::EMAIL,
    keys::SSL_ENABLED,
    keys::TEMPLATE,
    keys::RUNTIME_BACKEND,
    keys::WEB_ROOT,
    keys::NODE_VERSION,
    keys::SERVICES,
    keys::SCHEDULE_KEY,
    keys::RETENTION_DAYS_KEY,
    keys::PASSPHRASE_KEY,
    "PHP_VERSION",
    "PYTHON_VERSION",
    "RUBY_VERSION",
    "IS_STATIC",
    "INTERNAL_PORT",
];

#[derive(Clone, Debug, PartialEq)]
pub struct ParsedDotEnv {
    pub managed: BTreeMap<String, String>,
    pub applications: BTreeMap<String, String>,
    pub needs_rewrite: bool,
}

#[derive(Clone, Debug)]
pub struct LoadedLocal {
    pub environment: Bones,
    pub applications: BTreeMap<String, String>,
}

/// Parses the local environment grammar, including flat compatibility keys.
/// # Errors
/// Returns an error for malformed entries, reserved keys, unmatched blocks, or duplicates.
pub fn parse_dotenv(content: &str) -> Result<ParsedDotEnv> {
    let mut parsed = ParsedDotEnv { managed: BTreeMap::new(), applications: BTreeMap::new(), needs_rewrite: true };
    let mut in_block = false;
    let mut saw_block = false;
    for (number, raw) in content.split_inclusive('\n').chain((!content.ends_with('\n')).then_some("")).enumerate() {
        let line = raw.trim_end_matches('\n').trim_end_matches('\r');
        let trimmed = line.trim();
        if trimmed == BEGIN {
            if in_block {
                bail!("Nested managed block on line {}", number + 1);
            }
            in_block = true;
            saw_block = true;
            continue;
        }
        if trimmed == END {
            if !in_block {
                bail!("Unexpected managed block end on line {}", number + 1);
            }
            in_block = false;
            continue;
        }
        if in_block && (trimmed.is_empty() || trimmed.starts_with('#')) {
            continue;
        }
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((key, value)) = trimmed.split_once('=') else {
            bail!("Invalid .env entry on line {}", number + 1);
        };
        let key = key.trim();
        if !is_valid_env_name(key) {
            bail!("Invalid .env key on line {}", number + 1);
        }
        let value = strip_quotes(value.trim()).to_string();
        let (logical, managed) = if let Some(logical) = key.strip_prefix(MANAGED_PREFIX) {
            if !in_block && !MANAGED.contains(&logical) {
                bail!("Reserved .env key `{key}`; place it in the BonesDeploy managed block");
            }
            (logical.to_string(), true)
        } else if MANAGED.contains(&key) {
            (key.to_string(), true)
        } else if in_block {
            bail!("Managed block key `{key}` must start with BONES_");
        } else {
            (key.to_string(), false)
        };
        let target = if managed { &mut parsed.managed } else { &mut parsed.applications };
        if target.insert(logical, value).is_some() {
            bail!("Duplicate .env key `{key}` on line {}", number + 1);
        }
        if managed && !key.starts_with(MANAGED_PREFIX) {
            parsed.needs_rewrite = true;
        }
    }
    if in_block {
        bail!("Unclosed BonesDeploy managed block");
    }
    parsed.needs_rewrite |= !saw_block;
    Ok(parsed)
}

/// Loads local configuration and preserves application-owned key/value entries.
/// # Errors
/// Returns an error when the file is missing, malformed, or contains invalid configuration.
pub fn load_local(path: &Path) -> Result<LoadedLocal> {
    let content = fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let parsed = parse_dotenv(&content)?;
    let values = &parsed.managed;
    let project_name = values.get(keys::PROJECT_NAME).cloned().unwrap_or_default();
    let mut config = Bones::default();
    config.project_name = project_name.clone();
    config.remote_name = values.get(keys::REMOTE_NAME).cloned().unwrap_or_else(|| "production".into());
    config.ssh_user = values.get(keys::SSH_USER).cloned().unwrap_or_else(|| "root".into());
    config.host = values.get(keys::HOST).cloned().unwrap_or_default();
    config.port = values.get(keys::PORT).cloned().unwrap_or_else(|| "22".into());
    config.branch = values.get(keys::BRANCH).cloned().unwrap_or_else(|| "main".into());
    config.domain = values.get(keys::DOMAIN).cloned().unwrap_or_default();
    config.email = values.get(keys::EMAIL).cloned().unwrap_or_default();
    config.ssl_enabled = values.get(keys::SSL_ENABLED).is_some_and(|v| v == "true");
    config.runtime.template = values.get(keys::TEMPLATE).cloned().unwrap_or_default();
    config.runtime.web_root = values.get(keys::WEB_ROOT).cloned().unwrap_or_else(paths::default_web_root);
    config.runtime.node_version = values.get(keys::NODE_VERSION).cloned().unwrap_or_else(default_node_version);
    config.runtime.backend = match values.get(keys::RUNTIME_BACKEND).map_or("native", String::as_str) {
        "native" => RuntimeBackend::Native,
        "docker" => RuntimeBackend::Docker,
        value => bail!("Invalid RUNTIME_BACKEND: {value}"),
    };
    config.services.services = values
        .get(keys::SERVICES)
        .map(|v| v.split(',').filter(|s| !s.trim().is_empty()).map(|s| s.trim().into()).collect())
        .unwrap_or_default();
    config.backup.schedule =
        values.get(keys::SCHEDULE_KEY).cloned().unwrap_or_else(|| super::backup::BACKUP_SCHEDULE_DEFAULT.to_string());
    config.backup.retention_days = match values.get(keys::RETENTION_DAYS_KEY) {
        Some(value) => value.parse().with_context(|| format!("Invalid {} value: {value}", keys::RETENTION_DAYS_KEY))?,
        None => super::backup::BACKUP_RETENTION_DAYS_DEFAULT,
    };
    config.backup.passphrase = values.get(keys::PASSPHRASE_KEY).cloned().unwrap_or_default();
    for (key, value) in values {
        if !is_project_key(key) {
            config.runtime.extra.insert(key.to_ascii_lowercase(), parse_runtime_value(value));
        }
    }
    config.repo_path = default_repo_path_for(&project_name);
    config.project_root = paths::default_project_root_for(&project_name);
    validate_host(&config.host)?;
    validate_runtime(&config.runtime)?;
    validate_database_services(&config.services.services)?;
    validate_backup(&config.backup)?;
    Ok(LoadedLocal { environment: config, applications: parsed.applications })
}

/// Loads only the environment portion of a local file.
/// # Errors
/// Returns an error propagated from `load_local`.
pub fn load(path: &Path) -> Result<Bones> {
    Ok(load_local(path)?.environment)
}

/// Validates a complete local environment file.
/// # Errors
/// Returns an error when parsing fails.
pub fn validate_dotenv(content: &str) -> Result<()> {
    parse_dotenv(content).map(|_| ())
}

/// Removes managed keys from application values before production derivation.
/// The result cannot contain `BONES_*` keys because parsing reserves those names.
/// # Errors
/// This currently cannot fail; the result type is retained for the derivation boundary.
pub fn production_application_keys(parsed: &ParsedDotEnv) -> Result<BTreeMap<String, String>> {
    Ok(parsed
        .applications
        .iter()
        .filter(|(key, _)| !MANAGED.contains(&key.as_str()))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect())
}

/// Writes a normalized managed block atomically while retaining application text.
/// # Errors
/// Returns an error for invalid values or filesystem failures.
pub fn write_local_environment(config: &Bones, path: &Path) -> Result<()> {
    let old = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error).with_context(|| format!("Failed to read {}", path.display())),
    };
    parse_dotenv(&old)?;
    let mut application = String::new();
    let mut in_block = false;
    for raw in old.split_inclusive('\n') {
        let line = raw.trim_end_matches('\n').trim_end_matches('\r').trim();
        if line == BEGIN {
            in_block = true;
            continue;
        }
        if line == END {
            in_block = false;
            continue;
        }
        if in_block {
            continue;
        }
        if let Some((key, _)) = raw.trim_end_matches('\n').trim_end_matches('\r').trim().split_once('=') {
            if MANAGED.contains(&key.trim().strip_prefix(MANAGED_PREFIX).unwrap_or(key.trim())) {
                continue;
            }
        }
        application.push_str(raw);
    }
    if application.is_empty() {
        application = "# Local environment for the application.\n\n".into();
    }
    let application = application.trim_end_matches(['\r', '\n']);
    let mut output = format!("{application}\n\n{BEGIN}\n");
    let values = [
        (keys::PROJECT_NAME, config.project_name.clone()),
        (keys::REMOTE_NAME, config.remote_name.clone()),
        (keys::SSH_USER, config.ssh_user.clone()),
        (keys::HOST, config.host.clone()),
        (keys::PORT, config.port.clone()),
        (keys::BRANCH, config.branch.clone()),
        (keys::DOMAIN, config.domain.clone()),
        (keys::EMAIL, config.email.clone()),
        (keys::SSL_ENABLED, config.ssl_enabled.to_string()),
        (keys::TEMPLATE, config.runtime.template.clone()),
        (
            keys::RUNTIME_BACKEND,
            match config.runtime.backend {
                RuntimeBackend::Native => "native".into(),
                RuntimeBackend::Docker => "docker".into(),
            },
        ),
        (keys::WEB_ROOT, config.runtime.web_root.clone()),
        (keys::NODE_VERSION, config.runtime.node_version.clone()),
        (keys::SERVICES, config.services.services.join(",")),
        (keys::SCHEDULE_KEY, config.backup.schedule.clone()),
        (keys::RETENTION_DAYS_KEY, config.backup.retention_days.to_string()),
        (keys::PASSPHRASE_KEY, config.backup.passphrase.clone()),
    ];
    for (key, value) in values {
        append_value(&mut output, key, &value)?;
    }
    for (key, value) in &config.runtime.extra {
        append_value(
            &mut output,
            &key.to_ascii_uppercase(),
            &runtime_value_to_string(value)
                .with_context(|| format!("Runtime framework value `{key}` must be a scalar"))?,
        )?;
    }
    output.push_str(END);
    output.push('\n');
    atomic_write(path, output.as_bytes())
}

fn append_value(output: &mut String, key: &str, value: &str) -> Result<()> {
    if value.contains(['\n', '\r']) {
        bail!(".env values must not contain newlines");
    }
    output.push_str(MANAGED_PREFIX);
    output.push_str(key);
    output.push('=');
    output.push_str(&format_dotenv_value(value));
    output.push('\n');
    Ok(())
}
fn is_project_key(key: &str) -> bool {
    matches!(
        key,
        keys::PROJECT_NAME
            | keys::REMOTE_NAME
            | keys::SSH_USER
            | keys::HOST
            | keys::PORT
            | keys::BRANCH
            | keys::DOMAIN
            | keys::EMAIL
            | keys::SSL_ENABLED
            | keys::TEMPLATE
            | keys::RUNTIME_BACKEND
            | keys::WEB_ROOT
            | keys::NODE_VERSION
            | keys::SERVICES
            | keys::SCHEDULE_KEY
            | keys::RETENTION_DAYS_KEY
            | keys::PASSPHRASE_KEY
    )
}
fn parse_runtime_value(value: &str) -> toml::Value {
    match value {
        "true" => toml::Value::Boolean(true),
        "false" => toml::Value::Boolean(false),
        _ => toml::Value::String(value.into()),
    }
}
fn runtime_value_to_string(value: &toml::Value) -> Option<String> {
    match value {
        toml::Value::String(v) => Some(v.clone()),
        toml::Value::Integer(v) => Some(v.to_string()),
        toml::Value::Float(v) => Some(v.to_string()),
        toml::Value::Boolean(v) => Some(v.to_string()),
        toml::Value::Datetime(v) => Some(v.to_string()),
        toml::Value::Array(_) | toml::Value::Table(_) => None,
    }
}
fn is_valid_env_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else { return false };
    (first.is_ascii_alphabetic() || first == '_') && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
fn strip_quotes(value: &str) -> &str {
    let b = value.as_bytes();
    if b.len() >= 2 && ((b[0] == b'"' && b[b.len() - 1] == b'"') || (b[0] == b'\'' && b[b.len() - 1] == b'\'')) {
        &value[1..value.len() - 1]
    } else {
        value
    }
}
fn format_dotenv_value(value: &str) -> String {
    if value.trim() != value || value.starts_with(['"', '\'']) || value.ends_with(['"', '\'']) {
        format!("\"{value}\"")
    } else {
        value.into()
    }
}
