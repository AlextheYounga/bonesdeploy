use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BonesConfig {
    #[serde(default)]
    pub data: Data,
    #[serde(default)]
    pub permissions: Permissions,
    #[serde(default)]
    pub releases: Releases,
    #[serde(default)]
    pub runtime: Runtime,
}

pub struct Constants;

impl Constants {
    pub const BINARY_NAME: &str = "bonesremote";
    pub const SUDOERS_PATH: &str = "/etc/sudoers.d/bonesdeploy";
    pub const STAGED_RELEASE_FILE: &str = ".staged_release";
    pub const BUILD_DIR: &str = "build";
    pub const BUILD_WORKSPACE_DIR: &str = "workspace";
    pub const RUNTIME_DIR: &str = "runtime";
    pub const SHARED_DIR: &str = "shared";
    pub const CURRENT_LINK: &str = "current";
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Data {
    pub remote_name: String,
    pub project_name: String,
    pub host: String,
    pub port: String,
    pub git_dir: String,
    pub live_root: String,
    pub deploy_root: String,
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
            git_dir: String::new(),
            live_root: String::new(),
            deploy_root: String::new(),
            branch: "master".into(),
            deploy_on_push: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Releases {
    pub keep: usize,
    pub shared_paths: Vec<String>,
}

impl Default for Releases {
    fn default() -> Self {
        Self { keep: 5, shared_paths: Vec::new() }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Runtime {
    pub command: Vec<String>,
    pub working_dir: String,
    pub writable_paths: Vec<String>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self { command: Vec::new(), working_dir: ".".into(), writable_paths: Vec::new() }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Permissions {
    pub defaults: PermissionDefaults,
    pub paths: Vec<PathOverride>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct PermissionDefaults {
    pub deploy_user: String,
    pub service_user: String,
    pub group: String,
    pub dir_mode: String,
    pub file_mode: String,
}

impl Default for PermissionDefaults {
    fn default() -> Self {
        Self {
            deploy_user: "git".into(),
            service_user: String::new(),
            group: "www-data".into(),
            dir_mode: "750".into(),
            file_mode: "640".into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PathOverride {
    pub path: String,
    pub mode: String,
    #[serde(default)]
    pub recursive: bool,
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub path_type: Option<String>,
}

pub fn load(path: &Path) -> Result<BonesConfig> {
    let content = fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let mut config: BonesConfig =
        serde_yml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))?;
    apply_derived_defaults(&mut config);
    Ok(config)
}

// Fill in fields intentionally absent from bones.yaml so the rest of the app
// can read them as plain strings.
fn apply_derived_defaults(config: &mut BonesConfig) {
    let project_name = &config.data.project_name;

    if config.permissions.defaults.service_user.is_empty() {
        config.permissions.defaults.service_user = project_name.clone();
    }
    if config.data.live_root.is_empty() {
        config.data.live_root = format!("/var/www/{project_name}");
    }
    if config.data.deploy_root.is_empty() {
        config.data.deploy_root = format!("/srv/deployments/{project_name}");
    }
}
