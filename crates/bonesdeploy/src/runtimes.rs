use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde_json::Value;
use shared::config::Bones;
use shared::config::bonesinfra_input;

/// Shared question keys used by more than one template.
pub(crate) const IS_STATIC_KEY: &str = "is_static";

mod django;
mod laravel;
mod next;
mod nuxt;
mod rails;
mod svelte;
mod vue;

/// A promptable runtime question, lifted verbatim from bonesinfra's old
/// `runtime questions <fw>` output so agents and humans see the same shape
/// without a Python round-trip.
#[derive(Clone, Copy, Debug)]
pub struct Question {
    pub key: &'static str,
    pub label: &'static str,
    pub kind: QuestionKind,
}

#[derive(Clone, Copy, Debug)]
pub enum QuestionKind {
    Text { default: &'static str },
    Bool { default: bool },
    Choice { choices: &'static [&'static str], default: &'static str },
}

impl Question {
    pub fn default_value(&self) -> Value {
        match self.kind {
            QuestionKind::Text { default } | QuestionKind::Choice { default, .. } => Value::String(default.to_string()),
            QuestionKind::Bool { default } => Value::Bool(default),
        }
    }
}

/// Every promptable question for a runtime template. Empty for templates
/// that take no configuration (sveltekit, vue).
pub fn questions(template: &str) -> Result<&'static [Question]> {
    Ok(match template {
        "laravel" => laravel::questions(),
        "django" => django::questions(),
        "next" => next::questions(),
        "nuxt" => nuxt::questions(),
        "rails" => rails::questions(),
        "sveltekit" => svelte::questions(),
        "vue" => vue::questions(),
        other => bail!("unknown runtime template: {other}"),
    })
}

/// Validate non-interactive `--runtime-var` answers against a template's
/// question schema. Catches agent typos and bad values before they reach
/// `bones.toml`. Identity keys (`runtime_user`, `runtime_group`) are
/// injected later and skipped here.
pub fn validate_answers(template: &str, answers: &serde_json::Map<String, Value>) -> Result<()> {
    let schema = questions(template)?;
    for (key, value) in answers {
        if key == bonesinfra_input::RUNTIME_USER || key == bonesinfra_input::RUNTIME_GROUP {
            continue;
        }
        let Some(question) = schema.iter().find(|q| q.key == key.as_str()) else {
            bail!("unknown runtime var for {template}: {key}");
        };
        match (question.kind, value) {
            (QuestionKind::Text { .. }, Value::String(_)) | (QuestionKind::Bool { .. }, Value::Bool(_)) => {}
            (QuestionKind::Choice { choices, .. }, Value::String(s)) => {
                if !choices.contains(&s.as_str()) {
                    bail!("runtime var {key}={s} is not one of {choices:?} for {template}");
                }
            }
            _ => bail!("runtime var {key} has wrong type for {template}: {value}"),
        }
    }
    Ok(())
}

/// Apply template-specific post-scaffold configuration: runtime config
/// overrides (e.g. Next static → `web_root = "out"`) and `.env.prod.example`
/// substitutions (`<project>` and per-framework keys like Laravel's
/// `APP_URL`). Idempotent. Reads and writes the example file at most once.
pub fn configure(template: &str, cfg: &mut Bones, bones_dir: &Path) -> Result<()> {
    match template {
        "next" => next::configure(cfg),
        _ => {}
    }

    configure_env_example(template, cfg, bones_dir)
}

fn configure_env_example(template: &str, cfg: &Bones, bones_dir: &Path) -> Result<()> {
    let example = bones_dir.join("secrets/.env.prod.example");
    let content = match fs::read_to_string(&example) {
        Ok(content) => content,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error).with_context(|| format!("Failed to read {}", example.display())),
    };

    let mut configured = content.replace("<project>", &cfg.project_name);
    configured = match template {
        "laravel" => laravel::configure_env_example(&configured, cfg),
        _ => configured,
    };

    if configured != content {
        fs::write(&example, configured).with_context(|| format!("Failed to write {}", example.display()))?;
    }

    Ok(())
}

pub(crate) fn set_env_value(content: &str, key: &str, value: &str) -> String {
    let mut configured = content
        .lines()
        .map(|line| {
            if line.strip_prefix(key).is_some_and(|suffix| suffix.starts_with('=')) {
                format!("{key}={value}")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    if content.ends_with('\n') {
        configured.push('\n');
    }
    configured
}

#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::{Result, bail};
    use serde_json::{Map, Value, json};
    use shared::config::Bones;
    use tempfile::TempDir;

    use super::{QuestionKind, configure, questions, validate_answers};

    fn bones_with_runtime(template: &str, extra: Map<String, Value>) -> Result<Bones> {
        let mut config = Bones::default();
        config.project_name = String::from("atlas");
        let mut runtime = Map::new();
        runtime.insert("template".to_string(), Value::String(template.to_string()));
        runtime.insert("web_root".to_string(), Value::String("public".to_string()));
        for (k, v) in extra {
            runtime.insert(k, v);
        }
        config.runtime = serde_json::from_value(json!(runtime))?;
        Ok(config)
    }

    #[test]
    fn every_template_has_a_questions_function() -> Result<()> {
        for template in ["laravel", "django", "next", "nuxt", "rails", "sveltekit", "vue"] {
            questions(template)?;
        }
        Ok(())
    }

    #[test]
    fn sveltekit_and_vue_have_no_questions() -> Result<()> {
        assert!(questions("sveltekit")?.is_empty());
        assert!(questions("vue")?.is_empty());
        Ok(())
    }

    #[test]
    fn laravel_questions_include_supported_values() -> Result<()> {
        let qs = questions("laravel")?;
        assert_eq!(qs.len(), 1);
        assert_eq!(qs[0].key, "php_version");
        assert!(matches!(qs[0].kind, QuestionKind::Choice { default: "8.5", .. }));
        Ok(())
    }

    #[test]
    fn django_questions_include_supported_values() -> Result<()> {
        let qs = questions("django")?;
        assert_eq!(qs.len(), 1);
        assert_eq!(qs[0].key, "wsgi_module");
        assert!(matches!(qs[0].kind, QuestionKind::Text { default: "config.wsgi:application" }));
        Ok(())
    }

    #[test]
    fn rails_questions_include_supported_values() -> Result<()> {
        let qs = questions("rails")?;
        assert_eq!(qs.len(), 1);
        assert_eq!(qs[0].key, "rails_env");
        assert!(matches!(qs[0].kind, QuestionKind::Text { default: "production" }));
        Ok(())
    }

    #[test]
    fn validate_rejects_unknown_key() -> Result<()> {
        let mut answers = Map::new();
        answers.insert("php_verison".to_string(), Value::String("8.5".to_string()));
        match validate_answers("laravel", &answers) {
            Ok(()) => bail!("expected error for unknown key"),
            Err(err) => {
                let msg = format!("{err:#}");
                assert!(msg.contains("unknown runtime var"), "got: {msg}");
                assert!(msg.contains("php_verison"), "got: {msg}");
            }
        }
        Ok(())
    }

    #[test]
    fn validate_rejects_bad_choice() -> Result<()> {
        let mut answers = Map::new();
        answers.insert("php_version".to_string(), Value::String("8.6".to_string()));
        match validate_answers("laravel", &answers) {
            Ok(()) => bail!("expected error for bad choice"),
            Err(err) => assert!(format!("{err:#}").contains("not one of"), "got: {err:#}"),
        }
        Ok(())
    }

    #[test]
    fn validate_rejects_wrong_type() -> Result<()> {
        let mut answers = Map::new();
        answers.insert("php_version".to_string(), Value::Bool(true));
        match validate_answers("laravel", &answers) {
            Ok(()) => bail!("expected error for wrong type"),
            Err(err) => assert!(format!("{err:#}").contains("wrong type"), "got: {err:#}"),
        }
        Ok(())
    }

    #[test]
    fn validate_accepts_defaults() -> Result<()> {
        let schema = questions("laravel")?;
        let mut answers = Map::new();
        for q in schema {
            answers.insert(q.key.to_string(), q.default_value());
        }
        validate_answers("laravel", &answers)?;
        Ok(())
    }

    #[test]
    fn validate_skips_runtime_identity_keys() -> Result<()> {
        let mut answers = Map::new();
        answers.insert("runtime_user".to_string(), Value::String("atlas".to_string()));
        answers.insert("runtime_group".to_string(), Value::String("atlas".to_string()));
        validate_answers("next", &answers)?;
        Ok(())
    }

    #[test]
    fn configure_static_next_overrides_web_root() -> Result<()> {
        let temp = TempDir::new()?;
        let mut config =
            bones_with_runtime("next", [("is_static".to_string(), Value::Bool(true))].into_iter().collect())?;
        configure("next", &mut config, temp.path())?;
        assert_eq!(config.runtime.web_root, "out");
        Ok(())
    }

    #[test]
    fn configure_server_next_keeps_web_root() -> Result<()> {
        let temp = TempDir::new()?;
        let mut config =
            bones_with_runtime("next", [("is_static".to_string(), Value::Bool(false))].into_iter().collect())?;
        configure("next", &mut config, temp.path())?;
        assert_eq!(config.runtime.web_root, "public");
        Ok(())
    }

    #[test]
    fn configure_substitutes_project_in_env_example() -> Result<()> {
        let temp = TempDir::new()?;
        let secrets = temp.path().join("secrets");
        fs::create_dir(&secrets)?;
        let example = secrets.join(".env.prod.example");
        fs::write(&example, "DATABASE_URL=sqlite:////srv/sites/<project>/shared/database.sqlite\n")?;

        let mut config = bones_with_runtime("next", Map::new())?;
        configure("next", &mut config, temp.path())?;

        assert_eq!(fs::read_to_string(example)?, "DATABASE_URL=sqlite:////srv/sites/atlas/shared/database.sqlite\n");
        Ok(())
    }

    #[test]
    fn configure_laravel_substitutes_app_url_from_preview_domain() -> Result<()> {
        let temp = TempDir::new()?;
        let secrets = temp.path().join("secrets");
        fs::create_dir(&secrets)?;
        let example = secrets.join(".env.prod.example");
        fs::write(&example, "APP_URL=https://example.com\nDB=/srv/sites/<project>/shared/db\n")?;

        let mut config = bones_with_runtime("laravel", Map::new())?;
        config.preview_domain = String::from("atlas-203-0-113-10.nip.io");
        configure("laravel", &mut config, temp.path())?;

        assert_eq!(
            fs::read_to_string(example)?,
            "APP_URL=http://atlas-203-0-113-10.nip.io\nDB=/srv/sites/atlas/shared/db\n"
        );
        Ok(())
    }

    #[test]
    fn configure_laravel_without_preview_domain_leaves_app_url_alone() -> Result<()> {
        let temp = TempDir::new()?;
        let secrets = temp.path().join("secrets");
        fs::create_dir(&secrets)?;
        let example = secrets.join(".env.prod.example");
        fs::write(&example, "APP_URL=https://example.com\n")?;

        let mut config = bones_with_runtime("laravel", Map::new())?;
        configure("laravel", &mut config, temp.path())?;

        assert_eq!(fs::read_to_string(example)?, "APP_URL=https://example.com\n");
        Ok(())
    }
}
