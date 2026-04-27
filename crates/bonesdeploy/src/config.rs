use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, anyhow};
use saphyr::{LoadableYamlNode, Yaml};
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
    let yaml = parse_yaml_document(&content).with_context(|| format!("Failed to parse {}", path.display()))?;

    let data_section = yaml.as_mapping_get("data");
    let permissions_section = yaml.as_mapping_get("permissions");
    let defaults_section = permissions_section.and_then(|permissions| permissions.as_mapping_get("defaults"));
    let releases_section = yaml.as_mapping_get("releases");
    let runtime_section = yaml.as_mapping_get("runtime");
    let ssl_section = yaml.as_mapping_get("ssl");

    let mut config = BonesConfig {
        data: Data {
            remote_name: read_string_field(data_section, "remote_name", String::new()),
            project_name: read_string_field(data_section, "project_name", String::new()),
            host: read_string_field(data_section, "host", String::new()),
            port: read_string_field(data_section, "port", default_port()),
            git_dir: read_string_field(data_section, "git_dir", String::new()),
            live_root: read_string_field(data_section, "live_root", String::new()),
            deploy_root: read_string_field(data_section, "deploy_root", String::new()),
            branch: read_string_field(data_section, "branch", default_branch()),
            deploy_on_push: read_bool_field(data_section, "deploy_on_push", default_deploy_on_push()),
        },
        permissions: Permissions {
            defaults: PermissionDefaults {
                deploy_user: read_string_field(defaults_section, "deploy_user", default_deploy_user()),
                service_user: read_string_field(defaults_section, "service_user", default_service_user()),
                group: read_string_field(defaults_section, "group", default_group()),
                dir_mode: read_string_field(defaults_section, "dir_mode", default_dir_mode()),
                file_mode: read_string_field(defaults_section, "file_mode", default_file_mode()),
            },
            paths: read_path_overrides(permissions_section),
        },
        releases: Releases {
            keep: read_usize_field(releases_section, "keep", default_keep()),
            shared_paths: read_string_list_field(releases_section, "shared_paths"),
        },
        runtime: Runtime {
            command: read_string_list_field(runtime_section, "command"),
            working_dir: read_string_field(runtime_section, "working_dir", default_runtime_working_dir()),
            writable_paths: read_string_list_field(runtime_section, "writable_paths"),
        },
        ssl: Ssl {
            enabled: read_bool_field(ssl_section, "enabled", false),
            domain: read_string_field(ssl_section, "domain", String::new()),
            email: read_string_field(ssl_section, "email", String::new()),
        },
    };

    if config.permissions.defaults.service_user.is_empty() {
        config.permissions.defaults.service_user = config.data.project_name.clone();
    }

    Ok(config)
}

pub fn save(config: &BonesConfig, path: &Path) -> Result<()> {
    let mut content = String::new();
    append_data_section(&mut content, &config.data);
    append_permissions_section(&mut content, &config.permissions);
    append_releases_section(&mut content, &config.releases);
    append_runtime_section(&mut content, &config.runtime);
    append_ssl_section(&mut content, &config.ssl);

    fs::write(path, content).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

fn append_data_section(content: &mut String, data: &Data) {
    let _ = writeln!(content, "data:");
    let _ = writeln!(content, "  remote_name: {}", yaml_quote(&data.remote_name));
    let _ = writeln!(content, "  project_name: {}", yaml_quote(&data.project_name));
    let _ = writeln!(content, "  host: {}", yaml_quote(&data.host));
    let _ = writeln!(content, "  port: {}", yaml_quote(&data.port));
    let _ = writeln!(content, "  git_dir: {}", yaml_quote(&data.git_dir));
    let _ = writeln!(content, "  live_root: {}", yaml_quote(&data.live_root));
    let _ = writeln!(content, "  deploy_root: {}", yaml_quote(&data.deploy_root));
    let _ = writeln!(content, "  branch: {}", yaml_quote(&data.branch));
    let _ = writeln!(content, "  deploy_on_push: {}", data.deploy_on_push);
    content.push('\n');
}

fn append_permissions_section(content: &mut String, permissions: &Permissions) {
    let _ = writeln!(content, "permissions:");
    let _ = writeln!(content, "  defaults:");
    let _ = writeln!(content, "    deploy_user: {}", yaml_quote(&permissions.defaults.deploy_user));
    let _ = writeln!(content, "    service_user: {}", yaml_quote(&permissions.defaults.service_user));
    let _ = writeln!(content, "    group: {}", yaml_quote(&permissions.defaults.group));
    let _ = writeln!(content, "    dir_mode: {}", yaml_quote(&permissions.defaults.dir_mode));
    let _ = writeln!(content, "    file_mode: {}", yaml_quote(&permissions.defaults.file_mode));

    if permissions.paths.is_empty() {
        let _ = writeln!(content, "  paths: []");
        content.push('\n');
        return;
    }

    let _ = writeln!(content, "  paths:");
    for path in &permissions.paths {
        let _ = writeln!(content, "    - path: {}", yaml_quote(&path.path));
        let _ = writeln!(content, "      mode: {}", yaml_quote(&path.mode));
        let _ = writeln!(content, "      recursive: {}", path.recursive);
        if let Some(path_type) = &path.path_type {
            let _ = writeln!(content, "      type: {}", yaml_quote(path_type));
        }
    }
    content.push('\n');
}

fn append_releases_section(content: &mut String, releases: &Releases) {
    let _ = writeln!(content, "releases:");
    let _ = writeln!(content, "  keep: {}", releases.keep);

    if releases.shared_paths.is_empty() {
        let _ = writeln!(content, "  shared_paths: []");
        content.push('\n');
        return;
    }

    let _ = writeln!(content, "  shared_paths:");
    for shared_path in &releases.shared_paths {
        let _ = writeln!(content, "    - {}", yaml_quote(shared_path));
    }
    content.push('\n');
}

fn append_runtime_section(content: &mut String, runtime: &Runtime) {
    if is_default_runtime(runtime) {
        content.push_str(
            "# Optional runtime launcher settings (only needed for service/landlock-managed apps).\n\
# runtime:\n\
#   command:\n\
#     - '/usr/bin/node'\n\
#     - 'server.js'\n\
#   working_dir: '.'\n\
#   writable_paths: []\n\n",
        );
        return;
    }

    let _ = writeln!(content, "runtime:");
    if runtime.command.is_empty() {
        let _ = writeln!(content, "  command: []");
    } else {
        let _ = writeln!(content, "  command:");
        for command_part in &runtime.command {
            let _ = writeln!(content, "    - {}", yaml_quote(command_part));
        }
    }

    let _ = writeln!(content, "  working_dir: {}", yaml_quote(&runtime.working_dir));
    if runtime.writable_paths.is_empty() {
        let _ = writeln!(content, "  writable_paths: []");
        content.push('\n');
        return;
    }

    let _ = writeln!(content, "  writable_paths:");
    for writable_path in &runtime.writable_paths {
        let _ = writeln!(content, "    - {}", yaml_quote(writable_path));
    }
    content.push('\n');
}

fn append_ssl_section(content: &mut String, ssl: &Ssl) {
    let _ = writeln!(content, "ssl:");
    let _ = writeln!(content, "  enabled: {}", ssl.enabled);
    let _ = writeln!(content, "  domain: {}", yaml_quote(&ssl.domain));
    let _ = writeln!(content, "  email: {}", yaml_quote(&ssl.email));
}

fn parse_yaml_document(content: &str) -> Result<Yaml<'_>> {
    let documents = Yaml::load_from_str(content).map_err(|error| anyhow!(error))?;
    documents.into_iter().next().context("YAML document is empty")
}

fn yaml_quote(value: &str) -> String {
    let escaped = value.replace('\'', "''");
    format!("'{escaped}'")
}

fn read_path_overrides(permissions: Option<&Yaml<'_>>) -> Vec<PathOverride> {
    let Some(paths_node) = permissions.and_then(|node| node.as_mapping_get("paths")) else {
        return Vec::new();
    };

    let Some(paths) = paths_node.as_sequence() else {
        return Vec::new();
    };

    paths
        .iter()
        .filter_map(|path_node| {
            let path = read_string_field(Some(path_node), "path", String::new());
            let mode = read_string_field(Some(path_node), "mode", String::new());

            if path.is_empty() || mode.is_empty() {
                return None;
            }

            Some(PathOverride {
                path,
                mode,
                recursive: read_bool_field(Some(path_node), "recursive", false),
                path_type: read_optional_string_field(Some(path_node), "type"),
            })
        })
        .collect()
}

fn read_string_field(section: Option<&Yaml<'_>>, key: &str, default: String) -> String {
    section.and_then(|node| node.as_mapping_get(key)).and_then(value_to_string).unwrap_or(default)
}

fn read_optional_string_field(section: Option<&Yaml<'_>>, key: &str) -> Option<String> {
    section.and_then(|node| node.as_mapping_get(key)).and_then(value_to_string)
}

fn read_bool_field(section: Option<&Yaml<'_>>, key: &str, default: bool) -> bool {
    section.and_then(|node| node.as_mapping_get(key)).and_then(value_to_bool).unwrap_or(default)
}

fn read_usize_field(section: Option<&Yaml<'_>>, key: &str, default: usize) -> usize {
    section.and_then(|node| node.as_mapping_get(key)).and_then(value_to_usize).unwrap_or(default)
}

fn read_string_list_field(section: Option<&Yaml<'_>>, key: &str) -> Vec<String> {
    let Some(values) = section.and_then(|node| node.as_mapping_get(key)).and_then(Yaml::as_sequence) else {
        return Vec::new();
    };

    values.iter().filter_map(value_to_string).collect()
}

fn value_to_string(value: &Yaml<'_>) -> Option<String> {
    if let Some(string) = value.as_str() {
        return Some(string.to_string());
    }
    if let Some(integer) = value.as_integer() {
        return Some(integer.to_string());
    }
    if let Some(float) = value.as_floating_point() {
        return Some(float.to_string());
    }
    value.as_bool().map(|boolean| boolean.to_string())
}

fn value_to_bool(value: &Yaml<'_>) -> Option<bool> {
    if let Some(boolean) = value.as_bool() {
        return Some(boolean);
    }

    let text = value.as_str()?.trim();
    if text.eq_ignore_ascii_case("true") {
        return Some(true);
    }
    if text.eq_ignore_ascii_case("false") {
        return Some(false);
    }
    None
}

fn value_to_usize(value: &Yaml<'_>) -> Option<usize> {
    if let Some(integer) = value.as_integer() {
        return usize::try_from(integer).ok();
    }

    value.as_str()?.trim().parse::<usize>().ok()
}
