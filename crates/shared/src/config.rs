use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::paths;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct BonesConfig {
    pub remote_name: String,
    pub project_name: String,
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

impl Default for BonesConfig {
    fn default() -> Self {
        Self {
            remote_name: String::new(),
            project_name: String::new(),
            host: String::new(),
            port: "22".into(),
            repo_path: String::new(),
            project_root: String::new(),
            branch: "master".into(),
            preview_domain: String::new(),
            deploy_on_push: true,
            releases_keep: 5,
            ssl_enabled: false,
            domain: String::new(),
            email: String::new(),
        }
    }
}

impl BonesConfig {
    pub fn deployment_paths(&self, web_root: &str) -> crate::paths::DeploymentPaths {
        crate::paths::DeploymentPaths::new(&self.project_name, &self.repo_path, &self.project_root, web_root)
    }
}

pub fn default_deploy_user() -> String {
    paths::DEPLOY_USER.to_string()
}

pub fn runtime_user_for(project_name: &str) -> String {
    project_name.to_string()
}

pub fn runtime_group_for(project_name: &str) -> String {
    project_name.to_string()
}

pub fn release_group_for(project_name: &str) -> String {
    format!("{project_name}-release")
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
    pub path_type: PathType,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PathType {
    File,
    Dir,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Permissions {
    pub paths: Vec<PathOverride>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathOverride {
    pub path: String,
    pub mode: String,
    #[serde(default)]
    pub recursive: bool,
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub path_type: Option<PathType>,
}

pub fn default_repo_path_for(project_name: &str) -> String {
    paths::default_repo_path_for(project_name)
}

pub fn default_project_root_for(project_name: &str) -> String {
    paths::default_project_root_for(project_name)
}

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

const RUNTIME_TOML: &str = "runtime.toml";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuntimeConfig {
    #[serde(default = "paths::default_web_root")]
    pub web_root: String,
}

pub fn load_runtime_config(config_dir: &Path) -> Result<RuntimeConfig> {
    let path = config_dir.join(RUNTIME_TOML);
    if path.exists() {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        Ok(toml::from_str(&content)
            .with_context(|| format!("Failed to parse {}", path.display()))?)
    } else {
        Ok(RuntimeConfig { web_root: paths::default_web_root() })
    }
}

pub fn apply_derived_defaults(config: &mut BonesConfig) {
    let project_name = &config.project_name;

    if config.repo_path.is_empty() {
        config.repo_path = default_repo_path_for(project_name);
    }
    if config.project_root.is_empty() {
        config.project_root = default_project_root_for(project_name);
    }
    if config.preview_domain.is_empty() {
        config.preview_domain = default_preview_domain_for(project_name, &config.host);
    }
}

pub fn load(path: &Path) -> Result<BonesConfig> {
    let content = fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let mut config: BonesConfig = toml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))?;
    apply_derived_defaults(&mut config);
    Ok(config)
}
