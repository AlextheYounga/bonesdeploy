pub use shared::config::{Bones, load};

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

    #[test]
    fn load_derives_project_root_and_repo_path_from_project_name() -> Result<()> {
        let path = temp_file_path("bonesremote_config_derived_defaults");
        let toml = r#"
project_name = "acme"
host = "example.com"
"#;

        fs::write(&path, toml)?;
        let cfg = load(&path)?;
        fs::remove_file(&path).ok();

        assert_eq!(cfg.repo_path, paths::default_repo_path_for("acme"));
        assert_eq!(cfg.project_root, paths::default_project_root_for("acme"));
        Ok(())
    }

    #[test]
    fn load_preserves_explicit_repo_and_project_root() -> Result<()> {
        let path = temp_file_path("bonesremote_config_explicit_values");
        let toml = r#"
project_name = "acme"
repo_path = "/custom/repo.git"
project_root = "/custom/deploy"
"#;

        fs::write(&path, toml)?;
        let cfg = load(&path)?;
        fs::remove_file(&path).ok();

        assert_eq!(cfg.repo_path, "/custom/repo.git");
        assert_eq!(cfg.project_root, "/custom/deploy");
        Ok(())
    }

    #[test]
    fn load_uses_defaults_for_missing_fields() -> Result<()> {
        let path = temp_file_path("bonesremote_config_missing_fields");
        fs::write(&path, "")?;

        let cfg = load(&path)?;
        fs::remove_file(&path).ok();

        assert_eq!(cfg.port, "22");
        assert_eq!(cfg.branch, "master");
        assert_eq!(cfg.releases_keep, 5);
        Ok(())
    }

    #[test]
    fn load_fails_for_invalid_toml() -> Result<()> {
        let path = temp_file_path("bonesremote_config_invalid_toml");
        fs::write(&path, "[data\n")?;

        let result = load(&path);
        fs::remove_file(&path).ok();

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn load_fails_for_missing_file() {
        let path = temp_file_path("bonesremote_config_missing_file");
        let result = load(&path);
        assert!(result.is_err());
    }
}
