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
}

pub struct Constants;

impl Constants {
    pub const BINARY_NAME: &str = "bonesremote";
    pub const SUDOERS_PATH: &str = "/etc/sudoers.d/bonesdeploy";
    pub const STAGED_RELEASE_FILE: &str = ".staged_release";
    pub const BUILD_DIR: &str = "build";
    pub const BUILD_WORKSPACE_DIR: &str = "workspace";
    pub const RELEASES_DIR: &str = "releases";
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
    pub repo_path: String,
    pub project_root: String,
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Releases {
    pub keep: usize,
    pub shared_files: Vec<String>,
    pub shared_dirs: Vec<String>,
}

impl Default for Releases {
    fn default() -> Self {
        Self { keep: 5, shared_files: Vec::new(), shared_dirs: Vec::new() }
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
    if config.data.repo_path.is_empty() {
        config.data.repo_path = default_repo_path_for(project_name);
    }
    if config.data.project_root.is_empty() {
        config.data.project_root = default_project_root_for(project_name);
    }
    if config.data.web_root.is_empty() {
        config.data.web_root = default_web_root();
    }
}

pub fn default_repo_path_for(project_name: &str) -> String {
    format!("/home/git/{project_name}.git")
}

pub fn default_project_root_for(project_name: &str) -> String {
    format!("/srv/deployments/{project_name}")
}

pub fn default_web_root() -> String {
    String::from("public")
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{default_project_root_for, default_repo_path_for, default_web_root, load};

    fn temp_file_path(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
        std::env::temp_dir().join(format!("{prefix}_{}_{}.yaml", std::process::id(), nanos))
    }

    #[test]
    fn load_derives_service_user_project_root_repo_path_and_web_root() {
        let path = temp_file_path("bonesremote_config_derived_defaults");
        let yaml = r#"
data:
  project_name: acme
  host: example.com
"#;

        fs::write(&path, yaml).unwrap_or_else(|error| panic!("failed to write test config: {error}"));
        let cfg = load(&path).unwrap_or_else(|error| panic!("failed to load test config: {error}"));
        fs::remove_file(&path).ok();

        assert_eq!(cfg.permissions.defaults.service_user, "acme");
        assert_eq!(cfg.data.repo_path, default_repo_path_for("acme"));
        assert_eq!(cfg.data.project_root, default_project_root_for("acme"));
        assert_eq!(cfg.data.web_root, default_web_root());
    }

    #[test]
    fn load_preserves_explicit_service_user_and_paths() {
        let path = temp_file_path("bonesremote_config_explicit_values");
        let yaml = r#"
data:
  project_name: acme
  repo_path: /custom/repo.git
  project_root: /custom/deploy
  web_root: dist
permissions:
  defaults:
    service_user: web
"#;

        fs::write(&path, yaml).unwrap_or_else(|error| panic!("failed to write test config: {error}"));
        let cfg = load(&path).unwrap_or_else(|error| panic!("failed to load test config: {error}"));
        fs::remove_file(&path).ok();

        assert_eq!(cfg.permissions.defaults.service_user, "web");
        assert_eq!(cfg.data.repo_path, "/custom/repo.git");
        assert_eq!(cfg.data.project_root, "/custom/deploy");
        assert_eq!(cfg.data.web_root, "dist");
    }

    #[test]
    fn load_uses_defaults_for_missing_fields() {
        let path = temp_file_path("bonesremote_config_missing_fields");
        fs::write(&path, "{}\n").unwrap_or_else(|error| panic!("failed to write test config: {error}"));

        let cfg = load(&path).unwrap_or_else(|error| panic!("failed to load test config: {error}"));
        fs::remove_file(&path).ok();

        assert_eq!(cfg.data.port, "22");
        assert_eq!(cfg.data.branch, "master");
        assert_eq!(cfg.permissions.defaults.deploy_user, "git");
        assert_eq!(cfg.releases.keep, 5);
        assert!(cfg.releases.shared_files.is_empty());
        assert!(cfg.releases.shared_dirs.is_empty());
    }

    #[test]
    fn load_fails_for_invalid_yaml() {
        let path = temp_file_path("bonesremote_config_invalid_yaml");
        fs::write(&path, "data: [\n").unwrap_or_else(|error| panic!("failed to write test config: {error}"));

        let result = load(&path);
        fs::remove_file(&path).ok();

        assert!(result.is_err());
    }

    #[test]
    fn load_fails_for_missing_file() {
        let path = temp_file_path("bonesremote_config_missing_file");
        let result = load(&path);
        assert!(result.is_err());
    }

    #[test]
    fn load_keeps_default_service_user_when_project_name_is_empty() {
        let path = temp_file_path("bonesremote_config_empty_project");
        let yaml = r#"
data:
  project_name: ''
"#;

        fs::write(&path, yaml).unwrap_or_else(|error| panic!("failed to write test config: {error}"));
        let cfg = load(&path).unwrap_or_else(|error| panic!("failed to load test config: {error}"));
        fs::remove_file(&path).ok();

        assert_eq!(cfg.permissions.defaults.service_user, "");
    }
}
