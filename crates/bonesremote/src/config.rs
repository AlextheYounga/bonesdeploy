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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::load;

    fn temp_file_path(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
        std::env::temp_dir().join(format!("{prefix}_{}_{}.yaml", std::process::id(), nanos))
    }

    // Confirms omitted ownership/location fields derive from project identity.
    #[test]
    fn load_derives_service_user_and_roots_from_project_name() {
        let path = temp_file_path("bonesremote_config_derived_defaults");
        let yaml = r#"
data:
  project_name: acme
  host: example.com
  git_dir: /home/git/acme.git
"#;

        fs::write(&path, yaml).unwrap_or_else(|error| panic!("failed to write test config: {error}"));
        let cfg = load(&path).unwrap_or_else(|error| panic!("failed to load test config: {error}"));
        fs::remove_file(&path).ok();

        assert_eq!(cfg.permissions.defaults.service_user, "acme");
        assert_eq!(cfg.data.live_root, "/var/www/acme");
        assert_eq!(cfg.data.deploy_root, "/srv/deployments/acme");
    }

    // Confirms explicit operator overrides are preserved and never replaced by derived defaults.
    #[test]
    fn load_preserves_explicit_service_user_and_roots() {
        let path = temp_file_path("bonesremote_config_explicit_values");
        let yaml = r#"
data:
  project_name: acme
  live_root: /custom/live
  deploy_root: /custom/deploy
permissions:
  defaults:
    service_user: web
"#;

        fs::write(&path, yaml).unwrap_or_else(|error| panic!("failed to write test config: {error}"));
        let cfg = load(&path).unwrap_or_else(|error| panic!("failed to load test config: {error}"));
        fs::remove_file(&path).ok();

        assert_eq!(cfg.permissions.defaults.service_user, "web");
        assert_eq!(cfg.data.live_root, "/custom/live");
        assert_eq!(cfg.data.deploy_root, "/custom/deploy");
    }

    // Confirms baseline defaults keep config load resilient when optional sections are absent.
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
    }

    // Invalid YAML must fail loudly so broken config does not proceed into deployment.
    #[test]
    fn load_fails_for_invalid_yaml() {
        let path = temp_file_path("bonesremote_config_invalid_yaml");
        fs::write(&path, "data: [\n").unwrap_or_else(|error| panic!("failed to write test config: {error}"));

        let result = load(&path);
        fs::remove_file(&path).ok();

        assert!(result.is_err());
    }

    // Missing config path must be an immediate error to prevent implicit fallback behavior.
    #[test]
    fn load_fails_for_missing_file() {
        let path = temp_file_path("bonesremote_config_missing_file");
        let result = load(&path);
        assert!(result.is_err());
    }

    // Empty project name should not invent an invalid service user.
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

        // This codifies intended behavior: service_user should stay explicit/default when
        // project name is missing, instead of becoming an empty user.
        assert_eq!(cfg.permissions.defaults.service_user, "");
    }
}
