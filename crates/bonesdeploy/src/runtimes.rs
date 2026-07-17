use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use anyhow::{Context, Result};
use shared::config::Bones;

pub(crate) fn apply(template: &str, config: &mut Bones, bones_dir: &Path) -> Result<()> {
    if template == "next" && is_static_next(config) {
        config.runtime.web_root = String::from("out");
    }

    configure_environment_example(bones_dir, &config.project_name)
}

fn is_static_next(config: &Bones) -> bool {
    config.runtime.extra.get("is_static").is_some_and(|value| value.to_string() == "true")
}

fn configure_environment_example(bones_dir: &Path, project_name: &str) -> Result<()> {
    let example = bones_dir.join("secrets/.env.prod.example");
    let content = match fs::read_to_string(&example) {
        Ok(content) => content,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error).with_context(|| format!("Failed to read {}", example.display())),
    };

    if content.contains("<project>") {
        fs::write(&example, content.replace("<project>", project_name))
            .with_context(|| format!("Failed to write {}", example.display()))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::Result;
    use serde_json::json;
    use shared::config::Bones;
    use tempfile::TempDir;

    use super::apply;

    fn next_config(is_static: bool) -> Result<Bones> {
        let mut config = Bones::default();
        config.project_name = String::from("atlas");
        config.runtime = serde_json::from_value(json!({
            "template": "next",
            "web_root": "public",
            "is_static": is_static,
        }))?;
        Ok(config)
    }

    #[test]
    fn static_next_uses_static_output_and_project_environment_example() -> Result<()> {
        let temp = TempDir::new()?;
        let secrets = temp.path().join("secrets");
        fs::create_dir(&secrets)?;
        let example = secrets.join(".env.prod.example");
        fs::write(&example, "DATABASE_URL=sqlite:////srv/sites/<project>/shared/database.sqlite\n")?;

        let mut config = next_config(true)?;
        apply("next", &mut config, temp.path())?;

        assert_eq!(config.runtime.web_root, "out");
        assert_eq!(fs::read_to_string(example)?, "DATABASE_URL=sqlite:////srv/sites/atlas/shared/database.sqlite\n");
        Ok(())
    }

    #[test]
    fn server_next_keeps_its_server_output() -> Result<()> {
        let temp = TempDir::new()?;
        let mut config = next_config(false)?;

        apply("next", &mut config, temp.path())?;

        assert_eq!(config.runtime.web_root, "public");
        Ok(())
    }
}
