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
    pub const BONES_SITE_SETUP_PLAYBOOK: &'static str = ".bones/site/playbooks/setup.yml";
    pub const BONES_SITE_ROLES_DIR: &'static str = ".bones/site/roles";

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

    let yaml = serde_yml::to_string(&to_serialize).context("Failed to serialize bones config")?;
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use anyhow::Result;

    use super::{
        BonesConfig, Data, PermissionDefaults, Permissions, Releases, Ssl, default_deploy_root_for,
        default_live_root_for, load, save,
    };

    fn temp_path(file_name: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
        std::env::temp_dir().join(format!("bonesdeploy_config_test_{}_{}_{}", process::id(), nanos, file_name))
    }

    fn minimal_yaml(project_name: &str) -> String {
        format!(
            "data:\n  remote_name: production\n  project_name: {project_name}\n  host: deploy.example.com\n  port: \"22\"\n  git_dir: /home/git/{project_name}.git\n  branch: master\n  deploy_on_push: true\n"
        )
    }

    fn sample_config(project_name: &str) -> BonesConfig {
        BonesConfig {
            data: Data {
                remote_name: String::from("production"),
                project_name: String::from(project_name),
                host: String::from("deploy.example.com"),
                port: String::from("22"),
                git_dir: format!("/home/git/{project_name}.git"),
                live_root: default_live_root_for(project_name),
                deploy_root: default_deploy_root_for(project_name),
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
            releases: Releases { keep: 5, shared_paths: Vec::new() },
            ssl: Ssl::default(),
        }
    }

    // Confirms service user derives from project name when omitted to keep init output minimal.
    #[test]
    fn load_applies_default_service_user_from_project_name() -> Result<()> {
        let path = temp_path("service_user.yaml");
        fs::write(&path, minimal_yaml("lawsnipe"))?;

        let cfg = load(&path)?;
        assert_eq!(cfg.permissions.defaults.service_user, "lawsnipe");

        fs::remove_file(path)?;
        Ok(())
    }

    // Confirms live_root default matches documented project-scoped runtime location.
    #[test]
    fn load_applies_default_live_root_from_project_name() -> Result<()> {
        let path = temp_path("live_root.yaml");
        fs::write(&path, minimal_yaml("atlas"))?;

        let cfg = load(&path)?;
        assert_eq!(cfg.data.live_root, "/var/www/atlas");

        fs::remove_file(path)?;
        Ok(())
    }

    // Confirms deploy_root default matches documented project-scoped release location.
    #[test]
    fn load_applies_default_deploy_root_from_project_name() -> Result<()> {
        let path = temp_path("deploy_root.yaml");
        fs::write(&path, minimal_yaml("atlas"))?;

        let cfg = load(&path)?;
        assert_eq!(cfg.data.deploy_root, "/srv/deployments/atlas");

        fs::remove_file(path)?;
        Ok(())
    }

    // Ensures save omits derived values so config stays concise and portable across renames.
    #[test]
    fn save_omits_derived_live_and_deploy_roots() -> Result<()> {
        let config = sample_config("phoenix");
        let path = temp_path("save_derived_defaults.yaml");

        save(&config, &path)?;
        let content = fs::read_to_string(&path)?;

        assert!(!content.contains("live_root:"));
        assert!(!content.contains("deploy_root:"));

        fs::remove_file(path)?;
        Ok(())
    }

    // Ensures SSL settings are serialized so SSL setup can round-trip user intent.
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

    // Protects explicit user overrides so load never clobbers intentional non-default paths.
    #[test]
    fn load_preserves_explicit_live_and_deploy_root_overrides() -> Result<()> {
        let path = temp_path("overrides.yaml");
        let yaml = "data:\n  remote_name: production\n  project_name: app\n  host: deploy.example.com\n  port: \"22\"\n  git_dir: /home/git/app.git\n  live_root: /custom/live\n  deploy_root: /custom/deploy\n  branch: master\n  deploy_on_push: true\n";

        fs::write(&path, yaml)?;
        let cfg = load(Path::new(&path))?;

        assert_eq!(cfg.data.live_root, "/custom/live");
        assert_eq!(cfg.data.deploy_root, "/custom/deploy");

        fs::remove_file(path)?;
        Ok(())
    }
}
