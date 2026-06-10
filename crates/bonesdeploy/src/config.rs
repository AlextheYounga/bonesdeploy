use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use shared::paths;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BonesConfig {
    #[serde(default)]
    pub data: Data,
    #[serde(default)]
    pub permissions: Permissions,
    #[serde(default)]
    pub releases: Releases,
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
    pub const BONES_REMOTE_DIR: &'static str = ".bones/remote";
    pub const BONES_REMOTE_SETUP_PLAYBOOK: &'static str = ".bones/remote/playbooks/setup.yml";
    pub const BONES_REMOTE_ROLES_DIR: &'static str = ".bones/remote/roles";
    pub const BONES_REMOTE_APTFILE: &'static str = ".bones/remote/Aptfile";

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
    pub shared_files: Vec<String>,
    pub shared_dirs: Vec<String>,
}

impl Default for Releases {
    fn default() -> Self {
        Self { keep: 5, shared_files: Vec::new(), shared_dirs: Vec::new() }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
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

pub fn is_configured(config: &BonesConfig) -> bool {
    let d = &config.data;
    !d.remote_name.is_empty() && !d.project_name.is_empty() && !d.host.is_empty() && !d.repo_path.is_empty()
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

    let yaml = serde_yml::to_string(&to_serialize).context("Failed to serialize bones config")?;
    fs::write(path, yaml).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

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

fn hide_derived_defaults(config: &mut BonesConfig) {
    let project_name = &config.data.project_name;

    if config.data.repo_path == default_repo_path_for(project_name) {
        config.data.repo_path.clear();
    }
    if config.data.project_root == default_project_root_for(project_name) {
        config.data.project_root.clear();
    }
    if config.data.web_root == default_web_root() {
        config.data.web_root.clear();
    }
}

#[cfg(test)]
mod tests {
    use std::env::temp_dir;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use anyhow::Result;
    use shared::paths;

    use super::{
        BonesConfig, Data, PermissionDefaults, Permissions, Releases, Ssl, default_project_root_for,
        default_repo_path_for, default_web_root, load, save,
    };

    fn temp_path(file_name: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
        temp_dir().join(format!("bonesdeploy_config_test_{}_{}_{}", process::id(), nanos, file_name))
    }

    fn minimal_yaml(project_name: &str) -> String {
        format!(
            "data:\n  remote_name: production\n  project_name: {project_name}\n  host: deploy.example.com\n  port: \"22\"\n  repo_path: {}\n  branch: master\n  deploy_on_push: true\n",
            paths::default_repo_path_for(project_name)
        )
    }

    fn sample_config(project_name: &str) -> BonesConfig {
        BonesConfig {
            data: Data {
                remote_name: String::from("production"),
                project_name: String::from(project_name),
                host: String::from("deploy.example.com"),
                port: String::from("22"),
                repo_path: default_repo_path_for(project_name),
                project_root: default_project_root_for(project_name),
                web_root: default_web_root(),
                branch: String::from("master"),
                deploy_on_push: true,
            },
            permissions: Permissions {
                defaults: PermissionDefaults {
                    deploy_user: String::from("git"),
                    service_user: String::from(project_name),
                    group: String::from("www-data"),
                    dir_mode: String::from("750"),
                    file_mode: String::from("640"),
                },
                paths: Vec::new(),
            },
            releases: Releases { keep: 5, shared_files: Vec::new(), shared_dirs: Vec::new() },
            ssl: Ssl::default(),
        }
    }

    /// Applies the project name as the default service user.
    #[test]
    fn load_applies_default_service_user_from_project_name() -> Result<()> {
        let path = temp_path("service_user.yaml");
        fs::write(&path, minimal_yaml("lawsnipe"))?;

        let cfg = load(&path)?;
        assert_eq!(cfg.permissions.defaults.service_user, "lawsnipe");

        fs::remove_file(path)?;
        Ok(())
    }

    /// Derives the default repo path from the project name.
    #[test]
    fn load_applies_default_repo_path_from_project_name() -> Result<()> {
        let path = temp_path("repo_path.yaml");
        fs::write(&path, minimal_yaml("atlas"))?;

        let cfg = load(&path)?;
        assert_eq!(cfg.data.repo_path, paths::default_repo_path_for("atlas"));

        fs::remove_file(path)?;
        Ok(())
    }

    /// Derives the default project root from the project name.
    #[test]
    fn load_applies_default_project_root_from_project_name() -> Result<()> {
        let path = temp_path("project_root.yaml");
        fs::write(&path, minimal_yaml("atlas"))?;

        let cfg = load(&path)?;
        assert_eq!(cfg.data.project_root, paths::default_project_root_for("atlas"));

        fs::remove_file(path)?;
        Ok(())
    }

    /// Applies the default web root when not explicitly configured.
    #[test]
    fn load_applies_default_web_root() -> Result<()> {
        let path = temp_path("web_root.yaml");
        fs::write(&path, minimal_yaml("atlas"))?;

        let cfg = load(&path)?;
        assert_eq!(cfg.data.web_root, "public");

        fs::remove_file(path)?;
        Ok(())
    }

    /// Omits derived repo, project root, and web root fields when saving.
    #[test]
    fn save_omits_derived_repo_project_and_web_roots() -> Result<()> {
        let config = sample_config("phoenix");
        let path = temp_path("save_derived_defaults.yaml");

        save(&config, &path)?;
        let content = fs::read_to_string(&path)?;

        assert!(!content.contains("repo_path:"));
        assert!(!content.contains("project_root:"));
        assert!(!content.contains("web_root:"));

        fs::remove_file(path)?;
        Ok(())
    }

    /// Persists SSL settings (enabled, domain, email) when saving.
    #[test]
    fn save_persists_ssl_settings() -> Result<()> {
        let mut config = sample_config("phoenix");
        config.ssl =
            Ssl { enabled: true, domain: String::from("app.example.com"), email: String::from("ops@example.com") };

        let path = temp_path("save_ssl_settings.yaml");
        save(&config, &path)?;
        let content = fs::read_to_string(&path)?;

        assert!(content.contains("ssl:"));
        assert!(content.contains("enabled: true"));
        assert!(content.contains("domain: app.example.com"));
        assert!(content.contains("email: ops@example.com"));

        fs::remove_file(path)?;
        Ok(())
    }

    /// Preserves explicitly configured repo, project root, and web root overrides.
    #[test]
    fn load_preserves_explicit_repo_project_and_web_root_overrides() -> Result<()> {
        let path = temp_path("overrides.yaml");
        let yaml = format!(
            "data:\n  remote_name: production\n  project_name: app\n  host: deploy.example.com\n  port: \"22\"\n  repo_path: {}\n  project_root: /custom/deploy\n  web_root: dist\n  branch: master\n  deploy_on_push: true\n",
            paths::default_repo_path_for("app")
        );

        fs::write(&path, yaml)?;
        let cfg = load(Path::new(&path))?;

        assert_eq!(cfg.data.repo_path, paths::default_repo_path_for("app"));
        assert_eq!(cfg.data.project_root, "/custom/deploy");
        assert_eq!(cfg.data.web_root, "dist");

        fs::remove_file(path)?;
        Ok(())
    }
}
