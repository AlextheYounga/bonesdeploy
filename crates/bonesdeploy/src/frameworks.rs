use anyhow::{Result, bail};
use serde_json::Value;
use shared::config::{Bones, Framework};

/// Shared question keys used by more than one template.
pub(crate) const IS_STATIC_KEY: &str = "is_static";
const BUILD_ENV_HEADER: &str = "# Committed, non-secret values used while building this project.";

mod django;
mod laravel;
mod next;
mod nuxt;
mod rails;
mod sveltekit;
mod vue;

/// A promptable framework question, lifted verbatim from bonesinfra's old
/// `framework questions <fw>` output so agents and humans see the same shape
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

/// Every promptable question for a framework template. Empty for templates
/// that take no configuration (sveltekit, vue).
pub fn questions(template: &str) -> Result<&'static [Question]> {
    Ok(match template {
        "laravel" => laravel::questions(),
        "django" => django::questions(),
        "next" => next::questions(),
        "nuxt" => nuxt::questions(),
        "rails" => rails::questions(),
        "sveltekit" => sveltekit::questions(),
        "vue" => vue::questions(),
        other => bail!("unknown framework template: {other}"),
    })
}

/// Validate non-interactive `--framework-var` answers against a template's
/// question schema. Catches agent typos and bad values before they reach
/// `bones.toml`.
pub fn validate_answers(template: &str, answers: &serde_json::Map<String, Value>) -> Result<()> {
    let schema = questions(template)?;
    for (key, value) in answers {
        let Some(question) = schema.iter().find(|q| q.key == key.as_str()) else {
            bail!("unknown framework var for {template}: {key}");
        };
        match (question.kind, value) {
            (QuestionKind::Text { .. }, Value::String(_)) | (QuestionKind::Bool { .. }, Value::Bool(_)) => {}
            (QuestionKind::Choice { choices, .. }, Value::String(s)) => {
                if !choices.contains(&s.as_str()) {
                    bail!("framework var {key}={s} is not one of {choices:?} for {template}");
                }
            }
            _ => bail!("framework var {key} has wrong type for {template}: {value}"),
        }
    }
    Ok(())
}

/// Apply template-specific post-scaffold framework configuration.
pub fn configure(template: &str, cfg: &mut Bones) {
    match template {
        "next" => next::configure(cfg),
        "nuxt" => nuxt::configure(cfg),
        _ => {}
    }
}

pub fn environment_example(template: &str, project_name: &str, domain: &str, preview_domain: &str) -> Option<String> {
    let site_url = environment_url(domain, preview_domain);
    Some(match template {
        "django" => django::environment_example(project_name, &site_url),
        "laravel" => laravel::environment_example(project_name, &site_url),
        "next" => next::environment_example(project_name, &site_url),
        "nuxt" => nuxt::environment_example(project_name, &site_url),
        "rails" => rails::environment_example(project_name, &site_url),
        "sveltekit" => sveltekit::environment_example(project_name, &site_url),
        "vue" => vue::environment_example(project_name, &site_url),
        _ => return None,
    })
}

pub(crate) fn environment_url(domain: &str, preview_domain: &str) -> String {
    let host = if domain.is_empty() { preview_domain } else { domain };
    if host.is_empty() { String::new() } else { format!("https://{host}") }
}

pub(crate) fn build_environment_example(template: &str, framework: &Framework) -> Option<String> {
    Some(match template {
        "django" => django::build_environment_example(framework),
        "laravel" => laravel::build_environment_example(framework),
        "next" => next::build_environment_example(),
        "nuxt" => nuxt::build_environment_example(),
        "rails" => rails::build_environment_example(),
        "sveltekit" => sveltekit::build_environment_example(),
        "vue" => vue::build_environment_example(),
        _ => return None,
    })
}

pub(crate) fn join_env_lines(lines: &[&str]) -> String {
    format!("{}\n", lines.join("\n"))
}

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use anyhow::{Result, bail};
    use serde_json::{Map, Value, json};
    use shared::config::Bones;

    use super::{Framework, build_environment_example, configure, environment_example, questions, validate_answers};

    fn bones_with_framework(template: &str, extra: Map<String, Value>) -> Result<Bones> {
        let mut config = Bones::default();
        config.project_name = String::from("atlas");
        let mut framework = Map::new();
        framework.insert("template".to_string(), Value::String(template.to_string()));
        framework.insert("web_root".to_string(), Value::String("public".to_string()));
        for (k, v) in extra {
            framework.insert(k, v);
        }
        config.framework = serde_json::from_value(json!(framework))?;
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
    fn every_template_has_an_environment_example() {
        for template in ["laravel", "django", "next", "nuxt", "rails", "sveltekit", "vue"] {
            assert!(
                environment_example(template, "atlas", "", "atlas.example.com").is_some(),
                "missing environment example for {template}"
            );
        }
    }

    #[test]
    fn environment_examples_use_project_name_in_shared_paths() {
        let laravel = environment_example("laravel", "atlas", "example.com", "atlas.example.com")
            .expect("Laravel environment defaults");
        assert!(laravel.contains("/srv/sites/atlas/shared/storage"));
        assert!(laravel.contains("APP_URL=https://example.com"));
        assert!(!laravel.contains("<project>"));

        let django =
            environment_example("django", "atlas", "", "atlas.example.com").expect("Django environment defaults");
        assert!(django.contains("/srv/sites/atlas/shared/database.sqlite"));

        let rails = environment_example("rails", "atlas", "", "atlas.example.com").expect("Rails environment defaults");
        assert!(rails.contains("/srv/sites/atlas/shared/storage/production.sqlite3"));
    }

    #[test]
    fn laravel_build_environment_uses_selected_php_version() {
        let framework: Framework = serde_json::from_value(serde_json::json!({ "php_version": "8.3" })).unwrap();

        let environment = build_environment_example("laravel", &framework).expect("Laravel build environment");
        assert!(environment.contains("PHP_VERSION=8.3"));
        assert!(!environment.contains("PHP_VERSION=8.5"));
    }

    #[test]
    fn django_build_environment_uses_selected_python_version() {
        let framework: Framework = serde_json::from_value(serde_json::json!({ "python_version": "3.12" })).unwrap();

        let environment = build_environment_example("django", &framework).expect("Django build environment");
        assert!(environment.contains("PYTHON_VERSION=3.12"));
        assert!(!environment.contains("PYTHON_VERSION=3.14"));
    }

    #[test]
    fn validate_rejects_unknown_key() -> Result<()> {
        let mut answers = Map::new();
        answers.insert("php_verison".to_string(), Value::String("8.5".to_string()));
        match validate_answers("laravel", &answers) {
            Ok(()) => bail!("expected error for unknown key"),
            Err(err) => {
                let msg = format!("{err:#}");
                assert!(msg.contains("unknown framework var"), "got: {msg}");
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
    fn configure_static_next_overrides_web_root() -> Result<()> {
        let mut config =
            bones_with_framework("next", [("is_static".to_string(), Value::Bool(true))].into_iter().collect())?;
        configure("next", &mut config);
        assert_eq!(config.framework.web_root, "out");
        Ok(())
    }

    #[test]
    fn configure_static_nuxt_overrides_web_root() -> Result<()> {
        let mut config =
            bones_with_framework("nuxt", [("is_static".to_string(), Value::Bool(true))].into_iter().collect())?;
        configure("nuxt", &mut config);
        assert_eq!(config.framework.web_root, ".output/public");
        Ok(())
    }

    #[test]
    fn configure_server_next_keeps_web_root() -> Result<()> {
        let mut config =
            bones_with_framework("next", [("is_static".to_string(), Value::Bool(false))].into_iter().collect())?;
        configure("next", &mut config);
        assert_eq!(config.framework.web_root, "public");
        Ok(())
    }
}
