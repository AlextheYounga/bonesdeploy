use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BonesConfig {
    pub data: Data,
    #[serde(default)]
    pub permissions: Permissions,
    #[serde(default)]
    pub releases: Releases,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    #[serde(default)]
    pub remote_name: String,
    #[serde(default)]
    pub project_name: String,
    #[serde(default)]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: String,
    #[serde(default)]
    pub git_dir: String,
    #[serde(default)]
    pub live_root: String,
    #[serde(default)]
    pub deploy_root: String,
    #[serde(default = "default_branch")]
    pub branch: String,
    #[serde(default = "default_deploy_on_push")]
    pub deploy_on_push: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Releases {
    #[serde(default = "default_keep")]
    pub keep: usize,
    #[serde(default)]
    pub shared_paths: Vec<String>,
}

impl Default for Releases {
    fn default() -> Self {
        Self { keep: default_keep(), shared_paths: Vec::new() }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Permissions {
    #[serde(default)]
    pub defaults: PermissionDefaults,
    #[serde(default)]
    pub paths: Vec<PathOverride>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionDefaults {
    #[serde(default = "default_deploy_user")]
    pub deploy_user: String,
    #[serde(default = "default_service_user")]
    pub service_user: String,
    #[serde(default = "default_group")]
    pub group: String,
    #[serde(default = "default_dir_mode")]
    pub dir_mode: String,
    #[serde(default = "default_file_mode")]
    pub file_mode: String,
}

impl Default for PermissionDefaults {
    fn default() -> Self {
        Self {
            deploy_user: default_deploy_user(),
            service_user: default_service_user(),
            group: default_group(),
            dir_mode: default_dir_mode(),
            file_mode: default_file_mode(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PathOverride {
    pub path: String,
    pub mode: String,
    #[serde(default)]
    pub recursive: bool,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub path_type: Option<String>,
}

fn default_port() -> String {
    "22".into()
}
fn default_branch() -> String {
    "master".into()
}
fn default_deploy_on_push() -> bool {
    true
}
fn default_keep() -> usize {
    5
}
fn default_deploy_user() -> String {
    "git".into()
}
fn default_service_user() -> String {
    "applications".into()
}
fn default_group() -> String {
    "www-data".into()
}
fn default_dir_mode() -> String {
    "750".into()
}
fn default_file_mode() -> String {
    "640".into()
}

pub fn load(path: &Path) -> Result<BonesConfig> {
    let content = fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let config: BonesConfig =
        toml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))?;
    Ok(config)
}
