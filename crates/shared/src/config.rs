use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::paths;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Data {
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
    pub deploy_on_push: bool,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            remote_name: String::new(),
            project_name: String::new(),
            host: String::new(),
            port: "22".into(),
            repo_path: String::new(),
            project_root: String::new(),
            branch: "master".into(),
            deploy_on_push: true,
        }
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Releases {
    pub keep: usize,
}

impl Default for Releases {
    fn default() -> Self {
        Self { keep: 5 }
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

pub fn apply_derived_defaults(data: &mut Data) {
    let project_name = &data.project_name;

    if data.repo_path.is_empty() {
        data.repo_path = default_repo_path_for(project_name);
    }
    if data.project_root.is_empty() {
        data.project_root = default_project_root_for(project_name);
    }
}

impl Data {
    pub fn deployment_paths(&self, web_root: &str) -> crate::paths::DeploymentPaths {
        crate::paths::DeploymentPaths::new(&self.project_name, &self.repo_path, &self.project_root, web_root)
    }
}
