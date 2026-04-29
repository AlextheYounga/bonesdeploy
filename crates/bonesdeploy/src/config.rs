use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BonesConfig {
    #[serde(default)]
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
    pub const BONES_YAML: &'static str = ".bones/bones.yaml";
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
    pub const ASSET_SCRIPTS_DIR: &'static str = "scripts/";
    pub const PYTHON_BOOTSTRAP_SCRIPT_ASSET: &'static str = "scripts/bootstrap_python3.sh";
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Data {
    pub remote_name: String,
    pub project_name: String,
    pub host: String,
    pub port: String,
    pub git_dir: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub live_root: String,
    #[serde(skip_serializing_if = "String::is_empty")]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Ssl {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub email: String,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PathOverride {
    pub path: String,
    pub mode: String,
    #[serde(default)]
    pub recursive: bool,
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub path_type: Option<String>,
}

const RUNTIME_DOC_COMMENT: &str = "\
# Optional runtime launcher settings (only needed for service/landlock-managed apps).
# runtime:
#   command:
#     - '/usr/bin/node'
#     - 'server.js'
#   working_dir: '.'
#   writable_paths: []
";

fn is_default_runtime(runtime: &Runtime) -> bool {
    runtime == &Runtime::default()
}

pub fn is_configured(config: &BonesConfig) -> bool {
    let d = &config.data;
    !d.remote_name.is_empty() && !d.project_name.is_empty() && !d.host.is_empty() && !d.git_dir.is_empty()
}

pub fn default_live_root_for(project_name: &str) -> String {
    format!("/var/www/{project_name}")
}

pub fn default_deploy_root_for(project_name: &str) -> String {
    format!("/srv/deployments/{project_name}")
}

pub fn load(path: &Path) -> Result<BonesConfig> {
    let content = fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let mut config: BonesConfig =
        serde_yml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))?;
    apply_derived_defaults(&mut config);
    Ok(config)
}

pub fn save(config: &BonesConfig, path: &Path) -> Result<()> {
    let mut to_serialize = config.clone();
    hide_derived_defaults(&mut to_serialize);

    let mut yaml = serde_yml::to_string(&to_serialize).context("Failed to serialize bones config")?;
    if is_default_runtime(&config.runtime) {
        yaml.push('\n');
        yaml.push_str(RUNTIME_DOC_COMMENT);
    }

    fs::write(path, yaml).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

// Fill in fields intentionally absent from bones.yaml so the rest of the app
// can read them as plain strings.
fn apply_derived_defaults(config: &mut BonesConfig) {
    let project_name = &config.data.project_name;

    if config.permissions.defaults.service_user.is_empty() {
        config.permissions.defaults.service_user = project_name.clone();
    }
    if config.data.live_root.is_empty() {
        config.data.live_root = default_live_root_for(project_name);
    }
    if config.data.deploy_root.is_empty() {
        config.data.deploy_root = default_deploy_root_for(project_name);
    }
}

// Inverse of apply_derived_defaults: clear paths that match the project-derived
// defaults so a freshly saved bones.yaml stays free of redundant overrides.
fn hide_derived_defaults(config: &mut BonesConfig) {
    let project_name = &config.data.project_name;

    if config.data.live_root == default_live_root_for(project_name) {
        config.data.live_root.clear();
    }
    if config.data.deploy_root == default_deploy_root_for(project_name) {
        config.data.deploy_root.clear();
    }
}
