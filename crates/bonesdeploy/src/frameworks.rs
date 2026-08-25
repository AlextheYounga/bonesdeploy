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
    pub const ALL: &'static [Self] =
        &[Self::Django, Self::Laravel, Self::Next, Self::Nuxt, Self::Rails, Self::SvelteKit, Self::Vue, Self::Custom];

    pub fn parse(template: &str) -> Result<Self> {
        template.parse()
    }

    pub fn display_name(self) -> String {
        match self {
            Self::Next => String::from("Next.js"),
            Self::SvelteKit => String::from("SvelteKit"),
            other => {
                let wire = other.to_string();
                let mut chars = wire.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            }
        }
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

    pub fn runtime_defaults(self) -> Result<Option<serde_json::Map<String, Value>>> {
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

    pub fn environment_example(self, project_name: &str, domain: &str) -> Option<String> {
        let site_url = environment_url(domain);
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

    pub fn build_environment_example(self, runtime: &Runtime) -> Option<String> {
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
pub(crate) fn environment_url(domain: &str) -> String {
    if domain.is_empty() { String::new() } else { format!("https://{domain}") }
}

pub(crate) fn render_env_template(template: &str, replacements: &[(&str, &str)]) -> String {
    replacements.iter().fold(template.to_string(), |content, (key, value)| content.replace(key, value))
}
