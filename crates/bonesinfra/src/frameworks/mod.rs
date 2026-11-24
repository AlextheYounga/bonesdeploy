use anyhow::{Result, bail};

use crate::ProvisioningContext;

mod angular;
mod custom;
mod django;
mod laravel;
mod next;
mod nuxt;
mod rails;
mod sveltekit;
mod vue;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Language {
    Node,
    Python,
    Ruby,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Placeholder {
    Django { module_path: String },
    Next,
    Nuxt,
    Rails,
    SvelteKit,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServerPlan {
    pub name: &'static str,
    pub label: &'static str,
    pub language: Language,
    pub nginx_template: &'static str,
    pub apparmor_template: &'static str,
    pub placeholder: Placeholder,
    pub command: String,
    pub validation_command: Option<String>,
    pub exec_paths: Vec<String>,
    pub writable_paths: Vec<String>,
    pub shared_directories: &'static [&'static str],
    pub tcp_port: Option<u16>,
    pub apparmor_network: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimePlan {
    Custom,
    Static { root: &'static str, nginx_template: &'static str, placeholder_template: &'static str },
    Server(ServerPlan),
    Laravel,
}

/// Selects and validates the built-in framework runtime owned by this crate.
///
/// # Errors
/// Returns an error for an unknown framework or invalid typed framework value.
pub fn plan(context: &ProvisioningContext) -> Result<RuntimePlan> {
    match if context.framework.is_empty() { "custom" } else { &context.framework } {
        "angular" => Ok(angular::plan(context)),
        "custom" => Ok(custom::plan(context)),
        "django" => django::plan(context),
        "laravel" => laravel::plan(context),
        "next" => next::plan(context),
        "nuxt" => nuxt::plan(context),
        "rails" => rails::plan(context),
        "sveltekit" => sveltekit::plan(context),
        "vue" => Ok(vue::plan(context)),
        framework => bail!("unknown framework infrastructure: {framework}"),
    }
}

pub(super) fn bool_value(context: &ProvisioningContext, key: &str, default: bool) -> Result<bool> {
    match context.extra.get(key) {
        None => Ok(default),
        Some(value) => value.as_bool().ok_or_else(|| anyhow::anyhow!("{key} must be a boolean")),
    }
}

pub(super) fn string_value(context: &ProvisioningContext, key: &str, default: &str) -> Result<String> {
    match context.extra.get(key) {
        None => Ok(default.to_owned()),
        Some(value) => value.as_str().map(str::to_owned).ok_or_else(|| anyhow::anyhow!("{key} must be a string")),
    }
}

pub(super) fn port_value(context: &ProvisioningContext, key: &str, default: u16) -> Result<u16> {
    match context.extra.get(key) {
        None => Ok(default),
        Some(value) => value
            .as_u64()
            .and_then(|value| u16::try_from(value).ok())
            .filter(|value| *value != 0)
            .ok_or_else(|| anyhow::anyhow!("{key} must be an integer between 1 and 65535")),
    }
}

pub(super) fn numeric_version(value: &str, components: usize, key: &str) -> Result<()> {
    let parts = value.split('.').collect::<Vec<_>>();
    if parts.len() == components
        && parts.iter().all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return Ok(());
    }
    bail!("{key} must contain {components} numeric dot-separated components")
}
