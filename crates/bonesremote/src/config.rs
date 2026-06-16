use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use shared::config as shared_config;
use shared::paths;

pub use shared::config::Data;
pub use shared::config::Releases;

#[derive(Debug, Serialize, Deserialize)]
pub struct BonesConfig {
    #[serde(default)]
    pub data: Data,
    #[serde(default)]
    pub releases: Releases,
}

pub struct Constants;

impl Constants {
    pub const BINARY_NAME: &str = paths::BONESREMOTE_BINARY;
    pub const SUDOERS_PATH: &str = paths::SUDOERS_PATH;
    pub const STAGED_RELEASE_FILE: &str = paths::STAGED_RELEASE_FILE;
    pub const BUILD_DIR: &str = paths::BUILD_DIR;
}

pub fn load(path: &Path) -> Result<BonesConfig> {
    let content = fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let mut config: BonesConfig =
        toml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))?;
    shared_config::apply_derived_defaults(&mut config.data);
    Ok(config)
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use anyhow::Result;
    use shared::paths;

    use super::load;

    fn temp_file_path(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
        env::temp_dir().join(format!("{prefix}_{}_{}.toml", process::id(), nanos))
    }

    /// Derives repo path, project root, and web root from the project name.
    #[test]
    fn load_derives_project_root_repo_path_and_web_root_from_project_name() -> Result<()> {
        let path = temp_file_path("bonesremote_config_derived_defaults");
        let toml = r#"
[data]
project_name = "acme"
host = "example.com"
"#;

        fs::write(&path, toml)?;
        let cfg = load(&path)?;
        fs::remove_file(&path).ok();

        assert_eq!(cfg.data.repo_path, paths::default_repo_path_for("acme"));
        assert_eq!(cfg.data.project_root, paths::default_project_root_for("acme"));
        assert_eq!(cfg.data.web_root, paths::default_web_root());
        Ok(())
    }

    /// Preserves explicitly configured repo path, project root, and web root.
    #[test]
    fn load_preserves_explicit_repo_project_and_web_root() -> Result<()> {
        let path = temp_file_path("bonesremote_config_explicit_values");
        let toml = r#"
[data]
project_name = "acme"
repo_path = "/custom/repo.git"
project_root = "/custom/deploy"
web_root = "dist"
"#;

        fs::write(&path, toml)?;
        let cfg = load(&path)?;
        fs::remove_file(&path).ok();

        assert_eq!(cfg.data.repo_path, "/custom/repo.git");
        assert_eq!(cfg.data.project_root, "/custom/deploy");
        assert_eq!(cfg.data.web_root, "dist");
        Ok(())
    }

    /// Applies default values for port, branch, and releases when fields are missing.
    #[test]
    fn load_uses_defaults_for_missing_fields() -> Result<()> {
        let path = temp_file_path("bonesremote_config_missing_fields");
        fs::write(&path, "")?;

        let cfg = load(&path)?;
        fs::remove_file(&path).ok();

        assert_eq!(cfg.data.port, "22");
        assert_eq!(cfg.data.branch, "master");
        assert_eq!(cfg.releases.keep, 5);
        Ok(())
    }

    /// Returns an error when the config file contains invalid TOML.
    #[test]
    fn load_fails_for_invalid_toml() -> Result<()> {
        let path = temp_file_path("bonesremote_config_invalid_toml");
        fs::write(&path, "[data\n")?;

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
