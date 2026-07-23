use anyhow::{Result, bail};
use serde_json::Value;
use shared::config::Bones;
use shared::config::bonesinfra_input;

/// Shared question keys used by more than one template.
pub(crate) const IS_STATIC_KEY: &str = "is_static";
const BUILD_ENV_HEADER: &str = "# Committed, non-secret values used while building this project.";

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

/// Apply template-specific post-scaffold runtime configuration.
pub fn configure(template: &str, cfg: &mut Bones) {
    match template {
        "next" => next::configure(cfg),
        "nuxt" => nuxt::configure(cfg),
        _ => {}
    }
}

#[cfg_attr(not(test), expect(dead_code))]
pub fn environment_example(template: &str) -> Option<String> {
    Some(match template {
        "django" => django::environment_example(),
        "laravel" => laravel::environment_example(),
        "next" => next::environment_example(),
        "nuxt" => nuxt::environment_example(),
        "rails" => rails::environment_example(),
        "sveltekit" => svelte::environment_example(),
        "vue" => vue::environment_example(),
        _ => return None,
    })
}

pub(crate) fn build_environment_example(template: &str) -> Option<String> {
    Some(match template {
        "django" => django::build_environment_example(),
        "laravel" => laravel::build_environment_example(),
        "next" => next::build_environment_example(),
        "nuxt" => nuxt::build_environment_example(),
        "rails" => rails::build_environment_example(),
        "sveltekit" => svelte::build_environment_example(),
        "vue" => vue::build_environment_example(),
        _ => return None,
    })
}

pub(crate) fn join_env_lines(lines: &[&str]) -> String {
    format!("{}\n", lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use anyhow::{Result, bail};
    use serde_json::{Map, Value, json};
    use shared::config::Bones;

    use super::{configure, environment_example, questions, validate_answers};

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
    fn every_template_has_an_environment_example() {
        for template in ["laravel", "django", "next", "nuxt", "rails", "sveltekit", "vue"] {
            assert!(environment_example(template).is_some(), "missing environment example for {template}");
        }
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
        let mut config =
            bones_with_runtime("next", [("is_static".to_string(), Value::Bool(true))].into_iter().collect())?;
        configure("next", &mut config);
        assert_eq!(config.runtime.web_root, "out");
        Ok(())
    }

    #[test]
    fn configure_static_nuxt_overrides_web_root() -> Result<()> {
        let mut config =
            bones_with_runtime("nuxt", [("is_static".to_string(), Value::Bool(true))].into_iter().collect())?;
        configure("nuxt", &mut config);
        assert_eq!(config.runtime.web_root, ".output/public");
        Ok(())
    }

    #[test]
    fn configure_server_next_keeps_web_root() -> Result<()> {
        let mut config =
            bones_with_runtime("next", [("is_static".to_string(), Value::Bool(false))].into_iter().collect())?;
        configure("next", &mut config);
        assert_eq!(config.runtime.web_root, "public");
        Ok(())
    }
}
