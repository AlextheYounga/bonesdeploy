use anyhow::Result;
use bonesdeploy_core::paths;
use std::env;

pub use bonesdeploy_core::config::{Bones, load, save};

/// Resolves the SSH user for provisioning commands: `BONES_BOOTSTRAP_SSH_USER`
/// overrides the configured `ssh_user`; blank values fall back to `root`.
pub fn bootstrap_ssh_user(config: &Bones) -> String {
    if let Ok(env_user) = env::var("BONES_BOOTSTRAP_SSH_USER") {
        let trimmed = env_user.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    let trimmed = config.ssh_user.trim();
    if trimmed.is_empty() { String::from("root") } else { trimmed.to_string() }
}

pub fn default_project_root_for(project_name: &str) -> String {
    paths::default_project_root_for(project_name)
}

pub fn repo_directory_name() -> Result<String> {
    let cwd = env::current_dir()?;
    Ok(cwd.file_name().map_or_else(|| String::from("project"), |n| n.to_string_lossy().to_string()))
}

#[cfg(test)]
mod tests {
    use std::env::temp_dir;
    use std::fs;
    use std::path::PathBuf;
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use anyhow::Result;

    use super::{Bones, bootstrap_ssh_user, save};
    use bonesdeploy_core::config::{RuntimeBackend, load};

    fn temp_path(file_name: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
        temp_dir().join(format!("bonesdeploy_config_test_{}_{}_{}", process::id(), nanos, file_name))
    }

    fn sample_config(project_name: &str) -> Bones {
        let mut config = Bones::default();
        config.remote_name = String::from("production");
        config.project_name = String::from(project_name);
        config.host = String::from("deploy.example.com");
        config.port = String::from("22");
        config.branch = String::from("master");
        config
    }

    #[test]
    fn bootstrap_ssh_user_defaults_to_root() {
        let mut config = Bones::default();
        config.ssh_user = String::new();
        assert_eq!(bootstrap_ssh_user(&config), "root");
    }

    #[test]
    fn bootstrap_ssh_user_uses_config_value() {
        let mut config = Bones::default();
        config.ssh_user = String::from("ubuntu");
        assert_eq!(bootstrap_ssh_user(&config), "ubuntu");
    }

    #[test]
    fn bootstrap_ssh_user_trims_and_rejects_blank_values() {
        let mut config = Bones::default();
        config.ssh_user = String::from("   ");
        assert_eq!(bootstrap_ssh_user(&config), "root");

        config.ssh_user = String::from("  ubuntu  ");
        assert_eq!(bootstrap_ssh_user(&config), "ubuntu");
    }

    #[test]
    fn save_round_trips_dotenv_values() -> Result<()> {
        let mut config = sample_config("phoenix");
        config.ssl_enabled = true;
        config.domain = String::from("app.example.com");
        config.preview_domain = String::from("preview.example.com");
        config.email = String::from("ops@example.com");
        config.runtime.template = String::from("next");
        config.runtime.backend = RuntimeBackend::Docker;
        config.runtime.web_root = String::from("dist");
        config.services.services = vec![String::from("postgres"), String::from("redis")];

        let path = temp_path("save.env");
        save(&config, &path)?;
        let content = fs::read_to_string(&path)?;

        assert!(content.contains("SSL_ENABLED=true"));
        assert!(content.contains("DOMAIN=app.example.com"));
        assert!(content.contains("EMAIL=ops@example.com"));
        let loaded = load(&path)?;
        assert_eq!(loaded.project_name, "phoenix");
        assert_eq!(loaded.remote_name, "production");
        assert_eq!(loaded.ssh_user, "root");
        assert_eq!(loaded.host, "deploy.example.com");
        assert_eq!(loaded.port, "22");
        assert_eq!(loaded.branch, "master");
        assert!(loaded.ssl_enabled);
        assert_eq!(loaded.preview_domain, "preview.example.com");
        assert_eq!(loaded.runtime.template, "next");
        assert_eq!(loaded.runtime.backend, RuntimeBackend::Docker);
        assert_eq!(loaded.runtime.web_root, "dist");
        assert_eq!(loaded.services.services, ["postgres", "redis"]);

        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn save_writes_flat_local_input_file() -> Result<()> {
        let path = temp_path("flat.env");
        save(&sample_config("phoenix"), &path)?;
        let content = fs::read_to_string(&path)?;
        assert!(content.lines().all(|line| !line.starts_with('[')));

        fs::remove_file(path)?;
        Ok(())
    }
}
