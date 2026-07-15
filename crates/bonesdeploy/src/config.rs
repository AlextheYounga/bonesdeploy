use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::str;

use anyhow::{Context, Result};
use serde_json::{Map, Value};
use shared::config as shared_config;
use shared::config::bonesinfra_input;
use shared::paths;

pub use shared::config::{Bones, load};

pub fn is_configured(config: &Bones) -> bool {
    !config.remote_name.is_empty()
        && !config.project_name.is_empty()
        && !config.host.is_empty()
        && !config.repo_path.is_empty()
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

pub fn save(config: &Bones, path: &Path) -> Result<()> {
    let mut to_serialize = config.clone();
    shared_config::apply_derived_defaults(&mut to_serialize);

    let mut doc = if path.exists() {
        let existing = fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
        existing.parse::<toml_edit::DocumentMut>().with_context(|| format!("Failed to parse {}", path.display()))?
    } else {
        toml_edit::DocumentMut::new()
    };

    doc["remote_name"] = toml_edit::value(to_serialize.remote_name.as_str());
    doc[bonesinfra_input::PROJECT_NAME] = toml_edit::value(to_serialize.project_name.as_str());
    doc["host"] = toml_edit::value(to_serialize.host.as_str());
    doc["port"] = toml_edit::value(to_serialize.port.as_str());
    doc[bonesinfra_input::SSH_USER] = toml_edit::value(to_serialize.ssh_user.as_str());
    doc[bonesinfra_input::REPO_PATH] = toml_edit::value(to_serialize.repo_path.as_str());
    doc[bonesinfra_input::PROJECT_ROOT] = toml_edit::value(to_serialize.project_root.as_str());
    doc["branch"] = toml_edit::value(to_serialize.branch.as_str());
    doc[bonesinfra_input::PREVIEW_DOMAIN] = toml_edit::value(to_serialize.preview_domain.as_str());
    doc["deploy_on_push"] = toml_edit::value(to_serialize.deploy_on_push);
    let releases = i64::try_from(to_serialize.releases_keep).unwrap_or(i64::MAX);
    doc["releases"] = toml_edit::value(releases);
    doc["ssl_enabled"] = toml_edit::value(to_serialize.ssl_enabled);
    doc["domain"] = toml_edit::value(to_serialize.domain.as_str());
    doc["email"] = toml_edit::value(to_serialize.email.as_str());

    fs::write(path, doc.to_string()).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

pub fn save_runtime(template_bytes: &[u8], config: &Map<String, Value>, path: &Path) -> Result<()> {
    let template = str::from_utf8(template_bytes).context("Runtime template is not valid UTF-8")?;
    let mut doc: toml_edit::DocumentMut = template.parse().context("Failed to parse runtime template")?;

    for (key, value) in config {
        let Some(toml_val) = json_to_toml_item(value) else {
            continue;
        };
        doc[key.as_str()] = toml_val;
    }

    fs::write(path, doc.to_string()).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

fn json_to_toml_item(v: &Value) -> Option<toml_edit::Item> {
    match v {
        Value::String(s) => Some(toml_edit::value(s.as_str())),
        Value::Number(n) => n.as_i64().map(toml_edit::value),
        Value::Bool(b) => Some(toml_edit::value(*b)),
        _ => None,
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

    use super::{Bones, default_project_root_for, save};
    use shared::config::load;

    fn temp_path(file_name: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
        temp_dir().join(format!("bonesdeploy_config_test_{}_{}_{}", process::id(), nanos, file_name))
    }

    fn minimal_toml(project_name: &str) -> String {
        format!(
            "remote_name = \"production\"\nproject_name = \"{project_name}\"\nhost = \"deploy.example.com\"\nport = \"22\"\nrepo_path = \"{}\"\nbranch = \"master\"\ndeploy_on_push = true\n",
            paths::default_repo_path_for(project_name)
        )
    }

    fn sample_config(project_name: &str) -> Bones {
        Bones {
            remote_name: String::from("production"),
            project_name: String::from(project_name),
            host: String::from("deploy.example.com"),
            port: String::from("22"),
            repo_path: paths::default_repo_path_for(project_name),
            project_root: default_project_root_for(project_name),
            branch: String::from("master"),
            deploy_on_push: true,
            ..Default::default()
        }
    }

    #[test]
    fn load_applies_default_repo_path_from_project_name() -> Result<()> {
        let path = temp_path("repo_path.toml");
        fs::write(&path, minimal_toml("atlas"))?;

        let cfg = load(&path)?;
        assert_eq!(cfg.repo_path, paths::default_repo_path_for("atlas"));

        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn load_applies_default_project_root_from_project_name() -> Result<()> {
        let path = temp_path("project_root.toml");
        fs::write(&path, minimal_toml("atlas"))?;

        let cfg = load(&path)?;
        assert_eq!(cfg.project_root, paths::default_project_root_for("atlas"));

        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn save_includes_derived_repo_and_project_root() -> Result<()> {
        let config = sample_config("phoenix");
        let path = temp_path("save_derived_defaults.toml");

        save(&config, &path)?;
        let content = fs::read_to_string(&path)?;

        assert!(content.contains("repo_path ="), "save should include repo_path");
        assert!(content.contains("project_root ="), "save should include project_root");

        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn save_persists_ssl_settings() -> Result<()> {
        let mut config = sample_config("phoenix");
        config.ssl_enabled = true;
        config.domain = String::from("app.example.com");
        config.email = String::from("ops@example.com");

        let path = temp_path("save_ssl_settings.toml");
        save(&config, &path)?;
        let content = fs::read_to_string(&path)?;

        assert!(content.contains("ssl_enabled = true"));
        assert!(content.contains("domain = \"app.example.com\""));
        assert!(content.contains("email = \"ops@example.com\""));

        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn load_preserves_explicit_repo_and_project_root_overrides() -> Result<()> {
        let path = temp_path("overrides.toml");
        let toml = format!(
            "remote_name = \"production\"\nproject_name = \"app\"\nhost = \"deploy.example.com\"\nport = \"22\"\nrepo_path = \"{}\"\nproject_root = \"/custom/deploy\"\nbranch = \"master\"\ndeploy_on_push = false\n",
            paths::default_repo_path_for("app")
        );

        fs::write(&path, toml)?;
        let cfg = load(Path::new(&path))?;

        assert_eq!(cfg.repo_path, paths::default_repo_path_for("app"));
        assert_eq!(cfg.project_root, "/custom/deploy");

        fs::remove_file(path)?;
        Ok(())
    }
}
