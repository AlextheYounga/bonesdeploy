use anyhow::{Context, Result, anyhow};
use serde_json::Value;
use shared::config::{bonesinfra_input, runtime_group_for, runtime_user_for};

use super::{Args, RuntimeSelection};
use crate::infra::assets::runtimes as runtime_assets;
use crate::runtimes;
use crate::ui::prompts;

pub(super) fn collect_runtime_config(args: &Args, project_name: &str) -> Result<RuntimeSelection> {
    let template = resolve_template(args)?;

    let Some(template_name) = template else {
        let mut vars = runtime_assets::base_runtime_defaults()?;
        inject_runtime_identity(&mut vars, project_name);
        return Ok(RuntimeSelection { template: None, config: vars });
    };

    let defaults = runtime_assets::runtime_defaults(&template_name)
        .with_context(|| format!("Failed to load embedded defaults for template {template_name}"))?;
    let map = if args.non_interactive {
        collect_non_interactive_answers(&template_name, args, &defaults)?
    } else {
        collect_interactive_answers(&template_name, &defaults)?
    };
    let mut map = map;
    inject_runtime_identity(&mut map, project_name);
    Ok(RuntimeSelection { template: Some(template_name), config: map })
}

fn resolve_template(args: &Args) -> Result<Option<String>> {
    let cli_template = args.template.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty());
    if let Some(raw) = cli_template {
        return Ok(if raw.eq_ignore_ascii_case("none") { None } else { Some(raw.to_string()) });
    }
    if args.non_interactive {
        return Ok(None);
    }
    let available = runtime_assets::runtime_names();
    prompts::choose_template(&available)
}

fn collect_non_interactive_answers(
    template_name: &str,
    args: &Args,
    defaults: &serde_json::Map<String, Value>,
) -> Result<serde_json::Map<String, Value>> {
    let mut user_vars: serde_json::Map<String, Value> = serde_json::Map::new();
    for raw in &args.runtime_vars {
        let parsed = parse_runtime_var(raw)?;
        user_vars.insert(parsed.0, parsed.1);
    }
    runtimes::validate_answers(template_name, &user_vars)
        .with_context(|| format!("Invalid --runtime-var answers for {template_name}"))?;
    let mut merged = defaults.clone();
    for (key, value) in user_vars {
        merged.insert(key, value);
    }
    Ok(merged)
}

fn collect_interactive_answers(
    template_name: &str,
    defaults: &serde_json::Map<String, Value>,
) -> Result<serde_json::Map<String, Value>> {
    let questions = runtimes::questions(template_name)?;
    prompts::prompt_runtime_questions(questions, defaults)
}

fn parse_runtime_var(raw: &str) -> Result<(String, Value)> {
    let (key, value) = raw.split_once('=').ok_or_else(|| anyhow!("--runtime-var must be KEY=VALUE, got: {raw}"))?;
    let key = key.trim();
    if key.is_empty() {
        return Err(anyhow!("--runtime-var key is empty in: {raw}"));
    }
    let value = parse_runtime_value(value.trim());
    Ok((key.to_string(), value))
}

fn parse_runtime_value(raw: &str) -> Value {
    if raw.eq_ignore_ascii_case("true") {
        Value::Bool(true)
    } else if raw.eq_ignore_ascii_case("false") {
        Value::Bool(false)
    } else {
        Value::String(raw.to_string())
    }
}

fn inject_runtime_identity(vars: &mut serde_json::Map<String, Value>, project_name: &str) {
    vars.insert(bonesinfra_input::RUNTIME_USER.into(), Value::String(runtime_user_for(project_name)));
    vars.insert(bonesinfra_input::RUNTIME_GROUP.into(), Value::String(runtime_group_for(project_name)));
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    fn args_non_interactive(template: Option<&str>, runtime_vars: &[&str]) -> Args {
        Args {
            non_interactive: true,
            project_name: Some(String::from("atlas")),
            branch: None,
            remote: None,
            host: Some(String::from("deploy.example.com")),
            port: None,
            template: template.map(String::from),
            runtime_vars: runtime_vars.iter().map(|value| String::from(*value)).collect(),
        }
    }

    #[test]
    fn runtime_var_parses_bool_true() -> Result<()> {
        let (key, value) = parse_runtime_var("is_static=true")?;
        assert_eq!(key, "is_static");
        assert_eq!(value, Value::Bool(true));
        Ok(())
    }

    #[test]
    fn runtime_var_parses_bool_false_case_insensitive() -> Result<()> {
        let (key, value) = parse_runtime_var("is_static=FALSE")?;
        assert_eq!(key, "is_static");
        assert_eq!(value, Value::Bool(false));
        Ok(())
    }

    #[test]
    fn runtime_var_parses_string() -> Result<()> {
        let (key, value) = parse_runtime_var("php_version=8.5")?;
        assert_eq!(key, "php_version");
        assert_eq!(value, Value::String("8.5".to_string()));
        Ok(())
    }

    #[test]
    fn runtime_var_rejects_missing_equals() -> Result<()> {
        match parse_runtime_var("is_static") {
            Ok(_) => anyhow::bail!("expected error for missing equals"),
            Err(err) => assert!(err.to_string().contains("KEY=VALUE"), "got: {err}"),
        }
        Ok(())
    }

    #[test]
    fn runtime_var_rejects_empty_key() -> Result<()> {
        match parse_runtime_var("=value") {
            Ok(_) => anyhow::bail!("expected error for empty key"),
            Err(err) => assert!(err.to_string().contains("empty"), "got: {err}"),
        }
        Ok(())
    }

    #[test]
    fn validate_accepts_known_runtime_vars() -> Result<()> {
        let args = args_non_interactive(Some("laravel"), &["php_version=8.5"]);
        let project = "atlas";
        let selection = collect_runtime_config(&args, project)?;
        assert_eq!(selection.template.as_deref(), Some("laravel"));
        assert_eq!(selection.config.get("php_version"), Some(&Value::String("8.5".to_string())));
        Ok(())
    }

    #[test]
    fn validate_rejects_unknown_runtime_var() -> Result<()> {
        let args = args_non_interactive(Some("laravel"), &["php_verison=8.5"]);
        match collect_runtime_config(&args, "atlas") {
            Ok(_) => anyhow::bail!("expected error for unknown runtime var"),
            Err(err) => {
                let msg = format!("{err:#}");
                assert!(msg.contains("unknown runtime var"), "got: {msg}");
            }
        }
        Ok(())
    }

    #[test]
    fn template_none_uses_base_defaults() -> Result<()> {
        let args = args_non_interactive(Some("none"), &[]);
        let selection = collect_runtime_config(&args, "atlas")?;
        assert!(selection.template.is_none());
        assert!(selection.config.contains_key("web_root") || selection.config.is_empty());
        Ok(())
    }

    #[test]
    fn template_omitted_uses_base_defaults() -> Result<()> {
        let args = args_non_interactive(None, &[]);
        let selection = collect_runtime_config(&args, "atlas")?;
        assert!(selection.template.is_none());
        Ok(())
    }
}
