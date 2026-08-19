use std::fmt;
use std::str::FromStr;

use anyhow::{Result, bail};
use bonesdeploy_core::config::{Bones, Runtime, default_node_version};
use serde::Serialize;
use serde_json::Value;

/// Shared question keys used by more than one template.
pub(crate) const IS_STATIC_KEY: &str = "is_static";
mod django;
mod laravel;
mod next;
mod nuxt;
mod rails;
mod sveltekit;
mod vue;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Framework {
    Django,
    Laravel,
    Next,
    Nuxt,
    Rails,
    SvelteKit,
    Vue,
    Custom,
}

impl Framework {
    pub(crate) const ALL: &'static [Self] =
        &[Self::Django, Self::Laravel, Self::Next, Self::Nuxt, Self::Rails, Self::SvelteKit, Self::Vue, Self::Custom];

    pub fn parse(template: &str) -> Result<Self> {
        template.parse()
    }

    pub fn questions(self) -> &'static [Question] {
        match self {
            Self::Django => django::questions(),
            Self::Laravel => laravel::questions(),
            Self::Next => next::questions(),
            Self::Nuxt => nuxt::questions(),
            Self::Rails => rails::questions(),
            Self::SvelteKit => sveltekit::questions(),
            Self::Vue => vue::questions(),
            Self::Custom => &[],
        }
    }

    pub fn validate_answers(self, answers: &serde_json::Map<String, Value>) -> Result<()> {
        let schema = self.questions();
        for (key, value) in answers {
            let Some(question) = schema.iter().find(|q| q.key == key.as_str()) else {
                bail!("unknown framework var for {self}: {key}");
            };
            match (question.kind, value) {
                (QuestionKind::Text { .. }, Value::String(_)) | (QuestionKind::Bool { .. }, Value::Bool(_)) => {}
                (QuestionKind::Choice { choices, .. }, Value::String(s)) => {
                    if !choices.contains(&s.as_str()) {
                        bail!("framework var {key}={s} is not one of {choices:?} for {self}");
                    }
                }
                _ => bail!("framework var {key} has wrong type for {self}: {value}"),
            }
        }
        Ok(())
    }

    fn defaults(self) -> Option<FrameworkDefaults> {
        match self {
            Self::Django => Some(django::defaults()),
            Self::Laravel => Some(laravel::defaults()),
            Self::Next => Some(next::defaults()),
            Self::Nuxt => Some(nuxt::defaults()),
            Self::Rails => Some(rails::defaults()),
            Self::SvelteKit => Some(sveltekit::defaults()),
            Self::Vue => Some(vue::defaults()),
            Self::Custom => None,
        }
    }

    pub(crate) fn runtime_defaults(self) -> Result<Option<serde_json::Map<String, Value>>> {
        let Some(defaults) = self.defaults() else {
            return Ok(None);
        };
        let mut values = serde_json::Map::new();
        values.insert("template".into(), Value::String(defaults.template.into()));
        values.insert("web_root".into(), Value::String(defaults.web_root.into()));
        values.insert("node_version".into(), Value::String(default_node_version()));
        if let Some((name, version)) = defaults.language {
            values.insert(name.into(), Value::String(version.into()));
        }
        values.insert("permissions".into(), serde_json::to_value(defaults.permissions)?);
        Ok(Some(values))
    }

    pub fn configure(self, cfg: &mut Bones) {
        match self {
            Self::Next => next::configure(cfg),
            Self::Nuxt => nuxt::configure(cfg),
            _ => {}
        }
    }

    pub fn environment_example(self, project_name: &str, domain: &str, preview_domain: &str) -> Option<String> {
        let site_url = environment_url(domain, preview_domain);
        Some(match self {
            Self::Django => django::environment_example(project_name, &site_url),
            Self::Laravel => laravel::environment_example(project_name, &site_url),
            Self::Next => next::environment_example(project_name, &site_url),
            Self::Nuxt => nuxt::environment_example(project_name, &site_url),
            Self::Rails => rails::environment_example(project_name, &site_url),
            Self::SvelteKit => sveltekit::environment_example(project_name, &site_url),
            Self::Vue => vue::environment_example(project_name, &site_url),
            Self::Custom => return None,
        })
    }

    pub(crate) fn build_environment_example(self, runtime: &Runtime) -> Option<String> {
        Some(match self {
            Self::Django => django::build_environment_example(runtime),
            Self::Laravel => laravel::build_environment_example(runtime),
            Self::Next => next::build_environment_example(),
            Self::Nuxt => nuxt::build_environment_example(),
            Self::Rails => rails::build_environment_example(runtime),
            Self::SvelteKit => sveltekit::build_environment_example(),
            Self::Vue => vue::build_environment_example(),
            Self::Custom => return None,
        })
    }
}

impl fmt::Display for Framework {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Django => "django",
            Self::Laravel => "laravel",
            Self::Next => "next",
            Self::Nuxt => "nuxt",
            Self::Rails => "rails",
            Self::SvelteKit => "sveltekit",
            Self::Vue => "vue",
            Self::Custom => "custom",
        })
    }
}

impl FromStr for Framework {
    type Err = anyhow::Error;

    fn from_str(template: &str) -> Result<Self, Self::Err> {
        match template {
            "django" => Ok(Self::Django),
            "laravel" => Ok(Self::Laravel),
            "next" => Ok(Self::Next),
            "nuxt" => Ok(Self::Nuxt),
            "rails" => Ok(Self::Rails),
            "sveltekit" => Ok(Self::SvelteKit),
            "vue" => Ok(Self::Vue),
            "custom" => Ok(Self::Custom),
            other => bail!("unknown framework template: {other}"),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct FrameworkDefaults {
    pub template: &'static str,
    pub web_root: &'static str,
    pub language: Option<(&'static str, &'static str)>,
    pub permissions: &'static [PermissionDefault],
}

#[derive(Serialize)]
pub(crate) struct PermissionDefault {
    pub path: &'static str,
    #[serde(rename = "type")]
    pub permission_type: &'static str,
    pub mode: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recursive: Option<bool>,
}

pub(crate) const fn directory(path: &'static str, mode: u16, recursive: bool) -> PermissionDefault {
    PermissionDefault { path, permission_type: "dir", mode, recursive: Some(recursive) }
}

pub(crate) const fn file(path: &'static str, mode: u16) -> PermissionDefault {
    PermissionDefault { path, permission_type: "file", mode, recursive: None }
}

pub(crate) const fn non_recursive_file(path: &'static str, mode: u16) -> PermissionDefault {
    PermissionDefault { path, permission_type: "file", mode, recursive: Some(false) }
}

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
pub(crate) fn environment_url(domain: &str, preview_domain: &str) -> String {
    let host = if domain.is_empty() { preview_domain } else { domain };
    if host.is_empty() { String::new() } else { format!("https://{host}") }
}

pub(crate) fn render_env_template(template: &str, replacements: &[(&str, &str)]) -> String {
    replacements.iter().fold(template.to_string(), |content, (key, value)| content.replace(key, value))
}

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use anyhow::{Result, bail};
    use bonesdeploy_core::config::Bones;
    use serde_json::{Map, Value, json};

    use super::{Framework, Runtime};

    fn bones_with_runtime(template: &str, extra: Map<String, Value>) -> Result<Bones> {
        let mut config = Bones::default();
        config.project_name = String::from("atlas");
        let mut runtime_vars = Map::new();
        runtime_vars.insert("template".to_string(), Value::String(template.to_string()));
        runtime_vars.insert("web_root".to_string(), Value::String("public".to_string()));
        for (k, v) in extra {
            runtime_vars.insert(k, v);
        }
        config.runtime = serde_json::from_value(json!(runtime_vars))?;
        Ok(config)
    }

    #[test]
    fn framework_wire_values_parse_and_display() -> Result<()> {
        for wire in ["django", "laravel", "next", "nuxt", "rails", "sveltekit", "vue", "custom"] {
            assert_eq!(Framework::parse(wire)?.to_string(), wire);
        }
        assert!(Framework::parse("unknown").is_err());
        Ok(())
    }

    #[test]
    fn custom_is_the_empty_framework_fallback() -> Result<()> {
        let answers = serde_json::Map::new();
        assert!(Framework::Custom.questions().is_empty());
        assert!(Framework::Custom.validate_answers(&answers).is_ok());
        assert!(Framework::Custom.environment_example("atlas", "", "").is_none());
        assert!(Framework::Custom.build_environment_example(&Runtime::default()).is_none());
        assert!(Framework::Custom.runtime_defaults()?.is_none());
        Ok(())
    }

    #[test]
    fn environment_examples_use_project_name_in_shared_paths() {
        let laravel = Framework::Laravel
            .environment_example("atlas", "example.com", "atlas.example.com")
            .expect("Laravel environment defaults");
        assert!(laravel.contains("/srv/sites/atlas/shared/storage"));
        assert!(laravel.contains("APP_URL=https://example.com"));
        assert!(!laravel.contains("<project>"));

        let django = Framework::Django
            .environment_example("atlas", "", "atlas.example.com")
            .expect("Django environment defaults");
        assert!(django.contains("/srv/sites/atlas/shared/database.sqlite"));

        let rails =
            Framework::Rails.environment_example("atlas", "", "atlas.example.com").expect("Rails environment defaults");
        assert!(rails.contains("/srv/sites/atlas/shared/storage/production.sqlite3"));
    }

    #[test]
    fn laravel_build_environment_uses_selected_php_version() {
        let runtime: Runtime = serde_json::from_value(serde_json::json!({ "php_version": "8.3" })).unwrap();

        let environment = Framework::Laravel.build_environment_example(&runtime).expect("Laravel build environment");
        assert!(environment.contains("PHP_VERSION=8.3"));
        assert!(!environment.contains("PHP_VERSION=8.5"));
    }

    #[test]
    fn django_build_environment_uses_selected_python_version() {
        let runtime: Runtime = serde_json::from_value(serde_json::json!({ "python_version": "3.12" })).unwrap();

        let environment = Framework::Django.build_environment_example(&runtime).expect("Django build environment");
        assert!(environment.contains("PYTHON_VERSION=3.12"));
        assert!(!environment.contains("PYTHON_VERSION=3.14"));
    }

    #[test]
    fn rails_build_environment_uses_selected_ruby_version() {
        let runtime: Runtime = serde_json::from_value(serde_json::json!({ "ruby_version": "3.4" })).unwrap();

        let environment = Framework::Rails.build_environment_example(&runtime).expect("Rails build environment");
        assert!(environment.contains("RUBY_VERSION=3.4"));
        assert!(!environment.contains("RUBY_VERSION=3.3"));
    }

    #[test]
    fn validate_rejects_unknown_key() -> Result<()> {
        let mut answers = Map::new();
        answers.insert("php_verison".to_string(), Value::String("8.5".to_string()));
        match Framework::Laravel.validate_answers(&answers) {
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
        match Framework::Laravel.validate_answers(&answers) {
            Ok(()) => bail!("expected error for bad choice"),
            Err(err) => assert!(format!("{err:#}").contains("not one of"), "got: {err:#}"),
        }
        Ok(())
    }

    #[test]
    fn validate_rejects_wrong_type() -> Result<()> {
        let mut answers = Map::new();
        answers.insert("php_version".to_string(), Value::Bool(true));
        match Framework::Laravel.validate_answers(&answers) {
            Ok(()) => bail!("expected error for wrong type"),
            Err(err) => assert!(format!("{err:#}").contains("wrong type"), "got: {err:#}"),
        }
        Ok(())
    }

    #[test]
    fn validate_accepts_defaults() -> Result<()> {
        let schema = Framework::Laravel.questions();
        let mut answers = Map::new();
        for q in schema {
            answers.insert(q.key.to_string(), q.default_value());
        }
        Framework::Laravel.validate_answers(&answers)?;
        Ok(())
    }

    #[test]
    fn configure_static_next_overrides_web_root() -> Result<()> {
        let mut config =
            bones_with_runtime("next", [("is_static".to_string(), Value::Bool(true))].into_iter().collect())?;
        Framework::Next.configure(&mut config);
        assert_eq!(config.runtime.web_root, "out");
        Ok(())
    }

    #[test]
    fn configure_static_nuxt_overrides_web_root() -> Result<()> {
        let mut config =
            bones_with_runtime("nuxt", [("is_static".to_string(), Value::Bool(true))].into_iter().collect())?;
        Framework::Nuxt.configure(&mut config);
        assert_eq!(config.runtime.web_root, ".output/public");
        Ok(())
    }

    #[test]
    fn configure_server_next_keeps_web_root() -> Result<()> {
        let mut config =
            bones_with_runtime("next", [("is_static".to_string(), Value::Bool(false))].into_iter().collect())?;
        Framework::Next.configure(&mut config);
        assert_eq!(config.runtime.web_root, "public");
        Ok(())
    }
}
