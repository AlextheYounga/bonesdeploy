use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::result::Result as StdResult;

use anyhow::{bail, Context, Result};
use serde::de::MapAccess;
use serde::{de, Deserialize, Serialize};

use crate::paths;

#[path = "app.rs"]
mod app;
pub use app::App;

#[path = "validation.rs"]
mod validation;
pub use validation::validate_project_name;

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
    pub const RELEASE_GROUP: &str = "release_group";
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Bones {
    pub app: App,
    #[serde(rename = "build")]
    pub buildtime: Buildtime,
    pub runtime: Runtime,
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
    #[serde(default)]
    pub template: String,
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
    #[serde(default)]
    pub permissions: Option<toml::Value>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, toml::Value>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self {
            template: String::new(),
            web_root: paths::default_web_root(),
            runtime_user: String::new(),
            runtime_group: String::new(),
            release_group: String::new(),
            shared: Shared::default(),
            permissions: None,
            extra: BTreeMap::new(),
        }
    }
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

#[derive(Clone, Debug, Default, Serialize)]
pub struct Buildtime {
    #[serde(flatten)]
    pub extra: BTreeMap<String, String>,
}

impl<'de> Deserialize<'de> for Buildtime {
    fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct BuildtimeVisitor;

        impl<'de> de::Visitor<'de> for BuildtimeVisitor {
            type Value = Buildtime;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a [build] table")
            }

            fn visit_map<M>(self, mut access: M) -> StdResult<Buildtime, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut extra = BTreeMap::new();
                while let Some((key, value)) = access.next_entry::<String, toml::Value>()? {
                    if key == "vars" {
                        return Err(de::Error::custom(
                            "[build].vars has been removed. Use .env.build for build-time variables.",
                        ));
                    }
                    extra.insert(key, value.to_string());
                }
                Ok(Buildtime { extra })
            }
        }

        deserializer.deserialize_map(BuildtimeVisitor)
    }
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
    if config.repo_path.is_empty() {
        config.repo_path = default_repo_path_for(&project_name);
    }
    if config.project_root.is_empty() {
        config.project_root = paths::default_project_root_for(&project_name);
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
    Ok(config)
}

#[cfg(test)]
mod tests {
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
    fn buildtime_rejects_removed_vars_key() {
        let result: std::result::Result<Buildtime, _> = toml::from_str(r#"vars = ["A"]"#);
        assert!(result.is_err(), "[build].vars should be rejected");
        let err = result.unwrap_err().to_string();
        assert!(err.contains(".env.build"), "error should mention .env.build: {err}");
    }
}
