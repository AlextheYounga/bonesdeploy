use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use shared::paths;

#[derive(Debug, Serialize, Deserialize)]
pub struct BonesConfig {
    #[serde(default)]
    pub data: Data,
    #[serde(default)]
    pub releases: Releases,
    #[serde(default)]
    pub shared: Shared,
}

pub struct Constants;

impl Constants {
    pub const BINARY_NAME: &str = paths::BONESREMOTE_BINARY;
    pub const SUDOERS_PATH: &str = paths::SUDOERS_PATH;
    pub const STAGED_RELEASE_FILE: &str = paths::STAGED_RELEASE_FILE;
    pub const BUILD_DIR: &str = paths::BUILD_DIR;
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
}

impl Default for Releases {
    fn default() -> Self {
        Self { keep: 5 }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Shared {
    pub shared_files: Vec<String>,
    pub shared_dirs: Vec<String>,
}

pub fn load(path: &Path) -> Result<BonesConfig> {
    let content = fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let mut config: BonesConfig =
        serde_yml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))?;
    apply_derived_defaults(&mut config);
    Ok(config)
}

fn apply_derived_defaults(config: &mut BonesConfig) {
    let project_name = &config.data.project_name;

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
    paths::default_repo_path_for(project_name)
}

pub fn default_project_root_for(project_name: &str) -> String {
    paths::default_project_root_for(project_name)
}

pub fn default_web_root() -> String {
    paths::default_web_root()
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use anyhow::Result;

    use super::{default_project_root_for, default_repo_path_for, default_web_root, load};

    fn temp_file_path(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
        env::temp_dir().join(format!("{prefix}_{}_{}.yaml", process::id(), nanos))
    }

    /// Derives repo path, project root, and web root from the project name.
    #[test]
    fn load_derives_project_root_repo_path_and_web_root_from_project_name() -> Result<()> {
        let path = temp_file_path("bonesremote_config_derived_defaults");
        let yaml = r"
data:
  project_name: acme
  host: example.com
";

        fs::write(&path, yaml)?;
        let cfg = load(&path)?;
        fs::remove_file(&path).ok();

        assert_eq!(cfg.data.repo_path, default_repo_path_for("acme"));
        assert_eq!(cfg.data.project_root, default_project_root_for("acme"));
        assert_eq!(cfg.data.web_root, default_web_root());
        Ok(())
    }

    /// Preserves explicitly configured repo path, project root, and web root.
    #[test]
    fn load_preserves_explicit_repo_project_and_web_root() -> Result<()> {
        let path = temp_file_path("bonesremote_config_explicit_values");
        let yaml = r"
data:
  project_name: acme
  repo_path: /custom/repo.git
  project_root: /custom/deploy
  web_root: dist
";

        fs::write(&path, yaml)?;
        let cfg = load(&path)?;
        fs::remove_file(&path).ok();

        assert_eq!(cfg.data.repo_path, "/custom/repo.git");
        assert_eq!(cfg.data.project_root, "/custom/deploy");
        assert_eq!(cfg.data.web_root, "dist");
        Ok(())
    }

    /// Applies default values for port, branch, releases, and shared when fields are missing.
    #[test]
    fn load_uses_defaults_for_missing_fields() -> Result<()> {
        let path = temp_file_path("bonesremote_config_missing_fields");
        fs::write(&path, "{}\n")?;

        let cfg = load(&path)?;
        fs::remove_file(&path).ok();

        assert_eq!(cfg.data.port, "22");
        assert_eq!(cfg.data.branch, "master");
        assert_eq!(cfg.releases.keep, 5);
        assert!(cfg.shared.shared_files.is_empty());
        assert!(cfg.shared.shared_dirs.is_empty());
        Ok(())
    }

    /// Returns an error when the config file contains invalid YAML.
    #[test]
    fn load_fails_for_invalid_yaml() -> Result<()> {
        let path = temp_file_path("bonesremote_config_invalid_yaml");
        fs::write(&path, "data: [\n")?;

        let result = load(&path);
        fs::remove_file(&path).ok();

        assert!(result.is_err());
        Ok(())
    }

    /// Returns an error when the config file does not exist.
    #[test]
    fn load_fails_for_missing_file() {
        let path = temp_file_path("bonesremote_config_missing_file");
        let result = load(&path);
        assert!(result.is_err());
    }
}
