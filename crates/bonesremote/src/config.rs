use std::fs;
use std::path::Path;

use anyhow::{Context, Result, anyhow};
use saphyr::{LoadableYamlNode, Yaml};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BonesConfig {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Runtime {
    #[serde(default)]
    pub command: Vec<String>,
    #[serde(default = "default_runtime_working_dir")]
    pub working_dir: String,
    #[serde(default = "default_runtime_writable_paths")]
    pub writable_paths: Vec<String>,
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

pub fn load(path: &Path) -> Result<BonesConfig> {
    let content = fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let yaml = parse_yaml_document(&content).with_context(|| format!("Failed to parse {}", path.display()))?;

    let data_section = yaml.as_mapping_get("data");
    let permissions_section = yaml.as_mapping_get("permissions");
    let defaults_section = permissions_section.and_then(|permissions| permissions.as_mapping_get("defaults"));
    let releases_section = yaml.as_mapping_get("releases");
    let runtime_section = yaml.as_mapping_get("runtime");

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
    };

    if config.permissions.defaults.service_user.is_empty() {
        config.permissions.defaults.service_user = config.data.project_name.clone();
    }

    Ok(config)
}

fn parse_yaml_document(content: &str) -> Result<Yaml<'_>> {
    let documents = Yaml::load_from_str(content).map_err(|error| anyhow!(error))?;
    documents.into_iter().next().context("YAML document is empty")
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
