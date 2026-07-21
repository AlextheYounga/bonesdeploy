use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use shared::config as shared_config;
use shared::paths;

pub use shared::config::{Bones, load};

pub fn is_configured(config: &Bones) -> bool {
    !config.remote_name.is_empty()
        && !config.project_name.is_empty()
        && !config.host.is_empty()
        && !config.repo_path.is_empty()
}

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

    let serialized = toml::to_string_pretty(&to_serialize).context("Failed to serialize bones.toml")?;
    let content = annotate_sections(&compact_inline_table_arrays(&serialized)?);
    fs::write(path, content).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

fn compact_inline_table_arrays(content: &str) -> Result<String> {
    let mut document = content.parse::<toml_edit::DocumentMut>().context("Failed to parse serialized bones.toml")?;

    let Some(runtime) = document.get_mut("runtime").and_then(toml_edit::Item::as_table_mut) else {
        return Ok(document.to_string());
    };
    for key in ["permissions", "shared"] {
        let Some(item) =
            runtime.get_mut(key).and_then(toml_edit::Item::as_table_mut).and_then(|table| table.get_mut("paths"))
        else {
            continue;
        };
        item.make_value();
    }

    Ok(document.to_string())
}

fn annotate_sections(content: &str) -> String {
    let comments = [
        ("[app]", "# Project identity and deployment settings."),
        ("[app.server]", "# Remote server connection."),
        ("[app.dns]", "# Domains, email, and TLS."),
        ("[app.deploy]", "# Branch and deployment behavior."),
        ("[build]", "# Environment variables and constants injected during builds."),
        ("[runtime]", "# Framework runtime settings."),
        ("[runtime.permissions]", "# Release file permissions."),
        ("[runtime.shared]", "# Paths persisted in the shared release directory."),
    ];

    let mut output = String::new();
    for line in content.lines() {
        if let Some((_, comment)) = comments.iter().find(|(section, _)| *section == line) {
            output.push_str(comment);
            output.push('\n');
        }
        output.push_str(line);
        output.push('\n');
    }
    output
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

    use super::{Bones, bootstrap_ssh_user, default_project_root_for, save};
    use shared::config::load;

    fn temp_path(file_name: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
        temp_dir().join(format!("bonesdeploy_config_test_{}_{}_{}", process::id(), nanos, file_name))
    }

    fn minimal_toml(project_name: &str) -> String {
        format!(
            "[app]\nremote_name = \"production\"\nproject_name = \"{project_name}\"\nrepo_path = \"{}\"\n[app.server]\nhost = \"deploy.example.com\"\nport = \"22\"\n[app.deploy]\nbranch = \"master\"\ndeploy_on_push = true\n",
            paths::default_repo_path_for(project_name)
        )
    }

    fn sample_config(project_name: &str) -> Bones {
        let mut config = Bones::default();
        config.remote_name = String::from("production");
        config.project_name = String::from(project_name);
        config.host = String::from("deploy.example.com");
        config.port = String::from("22");
        config.repo_path = paths::default_repo_path_for(project_name);
        config.project_root = default_project_root_for(project_name);
        config.branch = String::from("master");
        config.deploy_on_push = true;
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
    fn save_adds_comments_to_nested_sections() -> Result<()> {
        let path = temp_path("section_comments.toml");
        save(&sample_config("phoenix"), &path)?;
        let content = fs::read_to_string(&path)?;
        assert!(content.contains("# Remote server connection.\n[app.server]"));
        assert!(content.contains("# Branch and deployment behavior.\n[app.deploy]"));
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn save_formats_permission_entries_as_inline_tables() -> Result<()> {
        let mut config = sample_config("phoenix");
        config.runtime.permissions = Some(toml::from_str(
            r#"paths = [
                { path = "*", type = "dir", mode = "750" },
                { path = "storage", type = "dir", mode = "770", recursive = true },
            ]"#,
        )?);

        let path = temp_path("inline_permissions.toml");
        save(&config, &path)?;
        let content = fs::read_to_string(&path)?;

        let document = content.parse::<toml_edit::DocumentMut>()?;
        let paths = document
            .get("runtime")
            .and_then(toml_edit::Item::as_table)
            .and_then(|runtime| runtime.get("permissions"))
            .and_then(toml_edit::Item::as_table)
            .and_then(|permissions| permissions.get("paths"))
            .and_then(toml_edit::Item::as_array);

        assert!(paths.is_some_and(|paths| paths.iter().all(toml_edit::Value::is_inline_table)), "{content}");
        assert!(!content.contains("[[runtime.permissions.paths]]"), "{content}");
        load(&path)?;

        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn load_preserves_explicit_repo_and_project_root_overrides() -> Result<()> {
        let path = temp_path("overrides.toml");
        let toml = format!(
            "[app]\nremote_name = \"production\"\nproject_name = \"app\"\nrepo_path = \"{}\"\nproject_root = \"/custom/deploy\"\n[app.server]\nhost = \"deploy.example.com\"\nport = \"22\"\n[app.deploy]\nbranch = \"master\"\ndeploy_on_push = false\n",
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
