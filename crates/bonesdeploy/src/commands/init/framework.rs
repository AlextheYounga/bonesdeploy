use anyhow::{Context, Result, anyhow};
use bonesdeploy_core::config::{DATABASE_SERVICES, validate_database_services};
use serde_json::Value;

use super::{Args, FrameworkSelection};
use crate::frameworks;
use crate::infra::assets::frameworks as framework_assets;
use crate::ui::prompts;

pub fn collect_framework_config(args: &Args) -> Result<FrameworkSelection> {
    let template = resolve_template(args)?;

    let Some(template_name) = template else {
        let vars = framework_assets::base_framework_defaults()?;
        return Ok(FrameworkSelection { template: None, config: vars });
    };

    let framework = frameworks::Framework::parse(&template_name)?;
    let defaults = framework_assets::framework_defaults(&template_name)
        .with_context(|| format!("Failed to load embedded defaults for template {template_name}"))?;
    let map = if args.non_interactive {
        collect_non_interactive_answers(framework, args, &defaults)?
    } else {
        collect_interactive_answers(framework, &defaults)?
    };
    Ok(FrameworkSelection { template: Some(template_name), config: map })
}

pub fn collect_database_services(args: &Args) -> Result<Vec<String>> {
    if args.non_interactive {
        validate_database_services(&args.services)?;
        return Ok(args.services.clone());
    }
    prompts::choose_services(DATABASE_SERVICES)
}

fn resolve_template(args: &Args) -> Result<Option<String>> {
    let cli_template = args.template.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty());
    if let Some(raw) = cli_template {
        return Ok(if raw.eq_ignore_ascii_case("none") { None } else { Some(raw.to_string()) });
    }
    if args.non_interactive {
        return Ok(None);
    }
    let available = framework_assets::framework_names();
    prompts::choose_template(&available)
}

fn collect_non_interactive_answers(
    framework: frameworks::Framework,
    args: &Args,
    defaults: &serde_json::Map<String, Value>,
) -> Result<serde_json::Map<String, Value>> {
    let mut user_vars: serde_json::Map<String, Value> = serde_json::Map::new();
    for raw in &args.framework_vars {
        let parsed = parse_framework_var(raw)?;
        user_vars.insert(parsed.0, parsed.1);
    }
    framework
        .validate_answers(&user_vars)
        .with_context(|| format!("Invalid --framework-var answers for {framework}"))?;
    let mut merged = defaults.clone();
    for (key, value) in user_vars {
        merged.insert(key, value);
    }
    Ok(merged)
}

fn collect_interactive_answers(
    framework: frameworks::Framework,
    defaults: &serde_json::Map<String, Value>,
) -> Result<serde_json::Map<String, Value>> {
    prompts::prompt_framework_questions(framework.questions(), defaults)
}

pub fn parse_framework_var(raw: &str) -> Result<(String, Value)> {
    let (key, value) = raw.split_once('=').ok_or_else(|| anyhow!("--framework-var must be KEY=VALUE, got: {raw}"))?;
    let key = key.trim();
    if key.is_empty() {
        return Err(anyhow!("--framework-var key is empty in: {raw}"));
    }
    let value = parse_framework_value(value.trim());
    Ok((key.to_string(), value))
}

fn parse_framework_value(raw: &str) -> Value {
    if raw.eq_ignore_ascii_case("true") {
        Value::Bool(true)
    } else if raw.eq_ignore_ascii_case("false") {
        Value::Bool(false)
    } else {
        Value::String(raw.to_string())
    }
}
