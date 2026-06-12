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
    pub web_root: String,
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
            web_root: String::new(),
            branch: "master".into(),
            deploy_on_push: true,
        }
    }
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
    pub shared_files: Vec<String>,
    pub shared_dirs: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Permissions {
    pub defaults: PermissionDefaults,
    pub paths: Vec<PathOverride>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct PermissionDefaults {
    pub dir_mode: String,
    pub file_mode: String,
}

impl Default for PermissionDefaults {
    fn default() -> Self {
        Self { dir_mode: "750".into(), file_mode: "640".into() }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathOverride {
    pub path: String,
    pub mode: String,
    #[serde(default)]
    pub recursive: bool,
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub path_type: Option<String>,
}

pub fn default_repo_path_for(project_name: &str) -> String {
    paths::default_repo_path_for(project_name)
}

pub fn default_project_root_for(project_name: &str) -> String {
    paths::default_project_root_for(project_name)
}

pub fn default_web_root() -> String {
    paths::default_web_root()
}

pub fn apply_derived_defaults(data: &mut Data) {
    let project_name = &data.project_name;

    if data.repo_path.is_empty() {
        data.repo_path = default_repo_path_for(project_name);
    }
    if data.project_root.is_empty() {
        data.project_root = default_project_root_for(project_name);
    }
    if data.web_root.is_empty() {
        data.web_root = default_web_root();
    }
}

pub fn hide_derived_defaults(data: &mut Data) {
    let project_name = &data.project_name;

    if data.repo_path == default_repo_path_for(project_name) {
        data.repo_path.clear();
    }
    if data.project_root == default_project_root_for(project_name) {
        data.project_root.clear();
    }
    if data.web_root == default_web_root() {
        data.web_root.clear();
    }
}
