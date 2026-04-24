use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BonesConfig {
    pub data: Data,
    #[serde(default)]
    pub permissions: Permissions,
    #[serde(default)]
    pub releases: Releases,
    #[serde(default, skip_serializing_if = "is_default_runtime")]
    pub runtime: Runtime,
    #[serde(default)]
    pub ssl: Ssl,
}

pub struct Constants;

impl Constants {
    pub const BONES_DIR: &'static str = ".bones";
    pub const BONES_TOML: &'static str = ".bones/bones.toml";
    pub const BONES_HOOKS_SCRIPT: &'static str = ".bones/hooks.sh";
    pub const BONES_HOOKS_DIR: &'static str = ".bones/hooks";
    pub const BONES_DEPLOYMENT_DIR: &'static str = ".bones/deployment";
    pub const BONES_SERVER_SETUP_PLAYBOOK: &'static str = ".bones/server/playbooks/setup.yml";
    pub const BONES_SERVER_ROLES_DIR: &'static str = ".bones/server/roles";

    pub const GIT_HOOKS_DIR: &'static str = ".git/hooks";
    pub const GIT_PRE_PUSH_HOOK_PATH: &'static str = ".git/hooks/pre-push";
    pub const PRE_PUSH_HOOK: &'static str = "pre-push";
    pub const PRE_PUSH_HOOK_TARGET: &'static str = "../../.bones/hooks/pre-push";

    pub const REMOTE_BONES_DIR: &'static str = "bones";
    pub const REMOTE_HOOKS_DIR: &'static str = "hooks";
    pub const PRE_RECEIVE_HOOK: &'static str = "pre-receive";
    pub const POST_RECEIVE_HOOK: &'static str = "post-receive";

    pub const ASSET_HOOKS_DIR: &'static str = "hooks/";
    pub const ASSET_DEPLOYMENT_DIR: &'static str = "deployment/";
    pub const POST_RECEIVE_HOOK_ASSET: &'static str = "hooks/post-receive";
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Releases {
    #[serde(default = "default_keep")]
    pub keep: usize,
    #[serde(default)]
    pub shared_paths: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Runtime {
    #[serde(default)]
    pub command: Vec<String>,
    #[serde(default = "default_runtime_working_dir")]
    pub working_dir: String,
    #[serde(default = "default_runtime_writable_paths")]
    pub writable_paths: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Ssl {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub email: String,
}

impl Default for Runtime {
    fn default() -> Self {
        Self {
            command: Vec::new(),
            working_dir: default_runtime_working_dir(),
            writable_paths: default_runtime_writable_paths(),
        }
    }
}

impl Default for Releases {
    fn default() -> Self {
        Self { keep: default_keep(), shared_paths: Vec::new() }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Permissions {
    #[serde(default)]
    pub defaults: PermissionDefaults,
    #[serde(default)]
    pub paths: Vec<PathOverride>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    String::new()
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
fn default_runtime_working_dir() -> String {
    ".".into()
}
fn default_runtime_writable_paths() -> Vec<String> {
    Vec::new()
}

fn is_default_runtime(runtime: &Runtime) -> bool {
    runtime == &Runtime::default()
}

pub fn is_configured(config: &BonesConfig) -> bool {
    let d = &config.data;
    !d.remote_name.is_empty()
        && !d.project_name.is_empty()
        && !d.host.is_empty()
        && !d.git_dir.is_empty()
        && !d.live_root.is_empty()
        && !d.deploy_root.is_empty()
}

pub fn load(path: &Path) -> Result<BonesConfig> {
    let content = fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let mut config: BonesConfig =
        toml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))?;

    if config.permissions.defaults.service_user.is_empty() {
        config.permissions.defaults.service_user = config.data.project_name.clone();
    }

    Ok(config)
}

pub fn save(config: &BonesConfig, path: &Path) -> Result<()> {
    let mut content = toml::to_string_pretty(config).context("Failed to serialize config")?;

    if !content.contains("[runtime]") {
        content.push_str(
            "\n# Optional runtime launcher settings (only needed for service/landlock-managed apps).\n\
# [runtime]\n\
# command = [\"/usr/bin/node\", \"server.js\"]\n\
# working_dir = \".\"\n\
# writable_paths = []\n",
        );
    }

    fs::write(path, content).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}
