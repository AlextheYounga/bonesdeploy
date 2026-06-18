use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use shared::config as shared_config;
use shared::paths;

pub use shared::config::Data;
pub use shared::config::Releases;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BonesConfig {
    #[serde(default)]
    pub data: Data,
    #[serde(default)]
    pub releases: Releases,
    #[serde(default)]
    pub ssl: Ssl,
}

pub struct Constants;
impl Constants {
    pub const BONES_DIR: &'static str = paths::LOCAL_BONES_DIR;
    pub const BONES_TOML: &'static str = paths::LOCAL_BONES_TOML;
    pub const BONES_HOOKS_SCRIPT: &'static str = paths::LOCAL_BONES_HOOKS_SCRIPT;
    pub const BONES_HOOKS_DIR: &'static str = paths::LOCAL_BONES_HOOKS_DIR;
    pub const BONES_DEPLOYMENT_DIR: &'static str = paths::LOCAL_BONES_DEPLOYMENT_DIR;
    pub const BONES_RUNTIME_TOML: &'static str = paths::LOCAL_BONES_RUNTIME_TOML;

    pub const GIT_HOOKS_DIR: &'static str = ".git/hooks";
    pub const GIT_PRE_PUSH_HOOK_PATH: &'static str = ".git/hooks/pre-push";
    pub const PRE_PUSH_HOOK: &'static str = "pre-push";
    pub const PRE_PUSH_HOOK_TARGET: &'static str = "../../.bones/hooks/pre-push";

    pub const REMOTE_BONES_DIR: &'static str = "bones";
    pub const REMOTE_HOOKS_DIR: &'static str = "hooks";

    pub const ASSET_HOOKS_DIR: &'static str = "hooks/";
    pub const ASSET_DEPLOYMENT_DIR: &'static str = "deployment/";
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

pub fn is_configured(config: &BonesConfig) -> bool {
    let d = &config.data;
    !d.remote_name.is_empty() && !d.project_name.is_empty() && !d.host.is_empty() && !d.repo_path.is_empty()
}

pub fn default_project_root_for(project_name: &str) -> String {
    paths::default_project_root_for(project_name)
}

pub fn bones_config_dir(project_name: &str) -> PathBuf {
    paths::bones_config_root().join(format!("{project_name}.bones"))
}

pub fn repo_directory_name() -> Result<String> {
    let cwd = env::current_dir()?;
    Ok(cwd.file_name().map_or_else(|| String::from("project"), |n| n.to_string_lossy().to_string()))
}

pub fn load(path: &Path) -> Result<BonesConfig> {
    let content = fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let mut config: BonesConfig =
        toml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))?;
    shared_config::apply_derived_defaults(&mut config.data);
    Ok(config)
}

pub fn save(config: &BonesConfig, path: &Path) -> Result<()> {
    let mut to_serialize = config.clone();
    shared_config::hide_derived_defaults(&mut to_serialize.data);

    let toml_str = toml::to_string(&to_serialize).context("Failed to serialize bones config")?;
    fs::write(path, toml_str).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

pub fn save_runtime(runtime: &Map<String, Value>, path: &Path) -> Result<()> {
    let toml_str = toml::to_string(runtime).context("Failed to serialize runtime config")?;
    fs::write(path, toml_str).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
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

    use super::{BonesConfig, Data, Releases, Ssl, default_project_root_for, load, save};

    fn temp_path(file_name: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
        temp_dir().join(format!("bonesdeploy_config_test_{}_{}_{}", process::id(), nanos, file_name))
    }

    fn minimal_toml(project_name: &str) -> String {
        format!(
            "[data]\nremote_name = \"production\"\nproject_name = \"{project_name}\"\nhost = \"deploy.example.com\"\nport = \"22\"\nrepo_path = \"{}\"\nbranch = \"master\"\ndeploy_on_push = true\n",
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
                repo_path: paths::default_repo_path_for(project_name),
                project_root: default_project_root_for(project_name),
                branch: String::from("master"),
                deploy_on_push: true,
                ..Default::default()
            },
            releases: Releases::default(),
            ssl: Ssl::default(),
        }
    }

    /// Derives the default repo path from the project name.
    #[test]
    fn load_applies_default_repo_path_from_project_name() -> Result<()> {
        let path = temp_path("repo_path.toml");
        fs::write(&path, minimal_toml("atlas"))?;

        let cfg = load(&path)?;
        assert_eq!(cfg.data.repo_path, paths::default_repo_path_for("atlas"));

        fs::remove_file(path)?;
        Ok(())
    }

    /// Derives the default project root from the project name.
    #[test]
    fn load_applies_default_project_root_from_project_name() -> Result<()> {
        let path = temp_path("project_root.toml");
        fs::write(&path, minimal_toml("atlas"))?;

        let cfg = load(&path)?;
        assert_eq!(cfg.data.project_root, paths::default_project_root_for("atlas"));

        fs::remove_file(path)?;
        Ok(())
    }

    /// Omits derived repo and project root fields when saving.
    #[test]
    fn save_omits_derived_repo_and_project_root() -> Result<()> {
        let config = sample_config("phoenix");
        let path = temp_path("save_derived_defaults.toml");

        save(&config, &path)?;
        let content = fs::read_to_string(&path)?;

        assert!(!content.contains("repo_path:"));
        assert!(!content.contains("project_root:"));

        fs::remove_file(path)?;
        Ok(())
    }

    /// Persists SSL settings (enabled, domain, email) when saving.
    #[test]
    fn save_persists_ssl_settings() -> Result<()> {
        let mut config = sample_config("phoenix");
        config.ssl =
            Ssl { enabled: true, domain: String::from("app.example.com"), email: String::from("ops@example.com") };

        let path = temp_path("save_ssl_settings.toml");
        save(&config, &path)?;
        let content = fs::read_to_string(&path)?;

        assert!(content.contains("[ssl]"));
        assert!(content.contains("enabled = true"));
        assert!(content.contains("domain = \"app.example.com\""));
        assert!(content.contains("email = \"ops@example.com\""));

        fs::remove_file(path)?;
        Ok(())
    }

    /// Preserves explicitly configured repo and project root overrides.
    #[test]
    fn load_preserves_explicit_repo_and_project_root_overrides() -> Result<()> {
        let path = temp_path("overrides.toml");
        let toml = format!(
            "[data]\nremote_name = \"production\"\nproject_name = \"app\"\nhost = \"deploy.example.com\"\nport = \"22\"\nrepo_path = \"{}\"\nproject_root = \"/custom/deploy\"\nbranch = \"master\"\ndeploy_on_push = true\n",
            paths::default_repo_path_for("app")
        );

        fs::write(&path, toml)?;
        let cfg = load(Path::new(&path))?;

        assert_eq!(cfg.data.repo_path, paths::default_repo_path_for("app"));
        assert_eq!(cfg.data.project_root, "/custom/deploy");

        fs::remove_file(path)?;
        Ok(())
    }
}
