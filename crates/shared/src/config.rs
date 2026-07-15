use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::paths;

/// Keys in the JSON object that bonesdeploy sends to bonesinfra.
pub mod bonesinfra_input {
    pub const SSH_PORT: &str = "ssh_port";
    pub const SSH_USER: &str = "ssh_user";
    pub const DEPLOY_USER: &str = "deploy_user";
    pub const PROJECT_ROOT: &str = "project_root";
    pub const RUNTIME_USER: &str = "runtime_user";
    pub const RUNTIME_GROUP: &str = "runtime_group";
    pub const RELEASE_GROUP: &str = "release_group";
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Bones {
    pub remote_name: String,
    pub project_name: String,
    pub ssh_user: String,
    pub host: String,
    pub port: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub repo_path: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub project_root: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub branch: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub preview_domain: String,
    pub deploy_on_push: bool,
    #[serde(rename = "releases")]
    pub releases_keep: usize,
    pub ssl_enabled: bool,
    pub domain: String,
    pub email: String,
}

impl Default for Bones {
    fn default() -> Self {
        Self {
            remote_name: String::new(),
            project_name: String::new(),
            ssh_user: String::from("root"),
            host: String::new(),
            port: "22".into(),
            repo_path: String::new(),
            project_root: String::new(),
            branch: "master".into(),
            preview_domain: String::new(),
            deploy_on_push: false,
            releases_keep: 5,
            ssl_enabled: false,
            domain: String::new(),
            email: String::new(),
        }
    }
}

impl Bones {}

#[must_use]
pub fn default_deploy_user() -> String {
    paths::DEPLOY_USER.to_string()
}

/// # Errors
///
/// Returns an error if the string cannot be parsed as a `u16` port number.
pub fn parse_port(port: &str) -> Result<u16> {
    port.parse().with_context(|| format!("Invalid port: {port}"))
}

/// # Errors
///
/// Returns an error if the host contains characters that are not ASCII
/// alphanumeric, dots, or dashes.
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
pub fn release_group_for(project_name: &str) -> String {
    format!("{project_name}-release")
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
    #[serde(default = "paths::default_web_root")]
    pub web_root: String,
    #[serde(default)]
    pub runtime_user: String,
    #[serde(default)]
    pub runtime_group: String,
    #[serde(default)]
    pub release_group: String,
    #[serde(default)]
    pub shared: Shared,
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

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Buildtime {
    #[serde(default)]
    pub vars: Vec<String>,
}

/// # Errors
///
/// Returns an error if the file exists but cannot be read or parsed.
pub fn load_buildtime(config_dir: &Path) -> Result<Option<Buildtime>> {
    let path = config_dir.join(paths::BUILDTIME_TOML);
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))?;
    let buildtime: Buildtime =
        toml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))?;
    Ok(Some(buildtime))
}

#[must_use]
pub fn extract_env_vars(env_content: &str, var_names: &[String]) -> Vec<(String, String)> {
    env_content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return None;
            }
            let (key, value) = trimmed.split_once('=')?;
            let key = key.trim();
            if !var_names.iter().any(|name| name == key) {
                return None;
            }
            let value = strip_quotes(value.trim());
            Some((key.to_string(), value.to_string()))
        })
        .collect()
}

fn strip_quotes(s: &str) -> &str {
    let bytes = s.as_bytes();
    if bytes.len() >= 2
        && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\''))
    {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

/// Loads the runtime configuration from a TOML file, falling back to defaults.
///
/// # Errors
///
/// Returns an error if the file exists but cannot be read or parsed.
pub fn load_runtime(config_dir: &Path) -> Result<Runtime> {
    let path = config_dir.join(paths::RUNTIME_TOML);
    if path.exists() {
        let content = fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))?;
        let runtime: Runtime =
            toml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))?;
        Ok(runtime)
    } else {
        Ok(Runtime {
            web_root: paths::default_web_root(),
            runtime_user: String::new(),
            runtime_group: String::new(),
            release_group: String::new(),
            shared: Shared::default(),
        })
    }
}

pub fn apply_derived_defaults(config: &mut Bones) {
    let project_name = &config.project_name;

    if config.ssh_user.is_empty() {
        config.ssh_user = String::from("root");
    }
    if config.repo_path.is_empty() {
        config.repo_path = default_repo_path_for(project_name);
    }
    if config.project_root.is_empty() {
        config.project_root = paths::default_project_root_for(project_name);
    }
    if config.preview_domain.is_empty() {
        config.preview_domain = default_preview_domain_for(project_name, &config.host);
    }
}

/// Loads and parses a `bones.toml` configuration file, applying derived defaults.
///
/// # Errors
///
/// Returns an error if the file cannot be read or the TOML is invalid.
pub fn load(path: &Path) -> Result<Bones> {
    let content = fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let mut config: Bones = toml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))?;
    apply_derived_defaults(&mut config);
    validate_host(&config.host)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use std::{env, fs};

    use toml::de::Error;

    use super::*;

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
    fn runtime_parses_shared_paths() -> Result<(), Error> {
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
    fn build_identity_helpers_are_derived_from_project_name() {
        assert_eq!(build_user_for("demo"), "demo-build");
        assert_eq!(build_group_for("demo"), "demo-build");
    }

    #[test]
    fn extract_env_vars_parses_unquoted_values() {
        let vars = extract_env_vars("KEY=hello\nOTHER=world", &["KEY".into()]);
        assert_eq!(vars, vec![("KEY".to_string(), "hello".to_string())]);
    }

    #[test]
    fn extract_env_vars_parses_double_quoted_values() {
        let vars = extract_env_vars(r#"KEY="hello world""#, &["KEY".into()]);
        assert_eq!(vars, vec![("KEY".to_string(), "hello world".to_string())]);
    }

    #[test]
    fn extract_env_vars_parses_single_quoted_values() {
        let vars = extract_env_vars("KEY='hello world'", &["KEY".into()]);
        assert_eq!(vars, vec![("KEY".to_string(), "hello world".to_string())]);
    }

    #[test]
    fn extract_env_vars_skips_comments_and_blank_lines() {
        let content = "# comment\n\nKEY=val\n  \nOTHER=other";
        let vars = extract_env_vars(content, &["KEY".into()]);
        assert_eq!(vars, vec![("KEY".to_string(), "val".to_string())]);
    }

    #[test]
    fn extract_env_vars_returns_only_requested_keys() {
        let content = "A=1\nB=2\nC=3";
        let vars = extract_env_vars(content, &["A".into(), "C".into()]);
        assert_eq!(vars, vec![("A".to_string(), "1".to_string()), ("C".to_string(), "3".to_string())]);
    }

    #[test]
    fn load_buildtime_returns_none_for_missing_file() {
        let result = load_buildtime(Path::new("/tmp/nonexistent_dir_for_test")).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn load_buildtime_parses_vars_array() {
        let dir = env::temp_dir().join("bones-buildtime-test");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("buildtime.toml"), "vars = [\"A\", \"B\"]").unwrap();

        let result = load_buildtime(&dir).unwrap().unwrap();
        assert_eq!(result.vars, vec!["A", "B"]);

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn load_buildtime_empty_vars_is_fine() {
        let dir = env::temp_dir().join("bones-buildtime-empty-test");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("buildtime.toml"), "vars = []").unwrap();

        let result = load_buildtime(&dir).unwrap().unwrap();
        assert!(result.vars.is_empty());

        fs::remove_dir_all(&dir).ok();
    }
}
