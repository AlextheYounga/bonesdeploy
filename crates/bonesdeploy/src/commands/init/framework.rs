use anyhow::{Context, Result, anyhow};
use bonesdeploy_core::config::{DATABASE_SERVICES, validate_database_services};
use serde_json::Value;

use super::{Args, FrameworkSelection};
use crate::frameworks;
use crate::infra::assets::frameworks as framework_assets;
use crate::ui::prompts;

pub(super) fn collect_framework_config(args: &Args) -> Result<FrameworkSelection> {
    let template = resolve_template(args)?;

    let Some(template_name) = template else {
        let vars = framework_assets::base_framework_defaults()?;
        return Ok(FrameworkSelection { template: None, config: vars });
    };

    let defaults = framework_assets::framework_defaults(&template_name)
        .with_context(|| format!("Failed to load embedded defaults for template {template_name}"))?;
    let map = if args.non_interactive {
        collect_non_interactive_answers(&template_name, args, &defaults)?
    } else {
        collect_interactive_answers(&template_name, &defaults)?
    };
    Ok(FrameworkSelection { template: Some(template_name), config: map })
}

pub(super) fn collect_database_services(args: &Args) -> Result<Vec<String>> {
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
    template_name: &str,
    args: &Args,
    defaults: &serde_json::Map<String, Value>,
) -> Result<serde_json::Map<String, Value>> {
    let mut user_vars: serde_json::Map<String, Value> = serde_json::Map::new();
    for raw in &args.framework_vars {
        let parsed = parse_framework_var(raw)?;
        user_vars.insert(parsed.0, parsed.1);
    }
    frameworks::validate_answers(template_name, &user_vars)
        .with_context(|| format!("Invalid --framework-var answers for {template_name}"))?;
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
    let questions = frameworks::questions(template_name)?;
    prompts::prompt_framework_questions(questions, defaults)
}

fn parse_framework_var(raw: &str) -> Result<(String, Value)> {
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

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    fn args_non_interactive(template: Option<&str>, framework_vars: &[&str]) -> Args {
        Args {
            non_interactive: true,
            project_name: Some(String::from("atlas")),
            branch: None,
            remote: None,
            host: Some(String::from("deploy.example.com")),
            port: None,
            template: template.map(String::from),
            runtime_backend: None,
            framework_vars: framework_vars.iter().map(|value| String::from(*value)).collect(),
            services: Vec::new(),
        }
    }

    #[test]
    fn framework_var_parses_bool_true() -> Result<()> {
        let (key, value) = parse_framework_var("is_static=true")?;
        assert_eq!(key, "is_static");
        assert_eq!(value, Value::Bool(true));
        Ok(())
    }

    #[test]
    fn framework_var_parses_bool_false_case_insensitive() -> Result<()> {
        let (key, value) = parse_framework_var("is_static=FALSE")?;
        assert_eq!(key, "is_static");
        assert_eq!(value, Value::Bool(false));
        Ok(())
    }

    #[test]
    fn framework_var_parses_string() -> Result<()> {
        let (key, value) = parse_framework_var("php_version=8.5")?;
        assert_eq!(key, "php_version");
        assert_eq!(value, Value::String("8.5".to_string()));
        Ok(())
    }

    #[test]
    fn framework_var_rejects_missing_equals() -> Result<()> {
        match parse_framework_var("is_static") {
            Ok(_) => anyhow::bail!("expected error for missing equals"),
            Err(err) => assert!(err.to_string().contains("KEY=VALUE"), "got: {err}"),
        }
        Ok(())
    }

    #[test]
    fn framework_var_rejects_empty_key() -> Result<()> {
        match parse_framework_var("=value") {
            Ok(_) => anyhow::bail!("expected error for empty key"),
            Err(err) => assert!(err.to_string().contains("empty"), "got: {err}"),
        }
        Ok(())
    }

    #[test]
    fn validate_accepts_known_framework_vars() -> Result<()> {
        let args = args_non_interactive(Some("laravel"), &["php_version=8.5"]);
        let selection = collect_framework_config(&args)?;
        assert_eq!(selection.template.as_deref(), Some("laravel"));
        assert_eq!(selection.config.get("php_version"), Some(&Value::String("8.5".to_string())));
        Ok(())
    }

    #[test]
    fn validate_rejects_unknown_framework_var() -> Result<()> {
        let args = args_non_interactive(Some("laravel"), &["php_verison=8.5"]);
        match collect_framework_config(&args) {
            Ok(_) => anyhow::bail!("expected error for unknown framework var"),
            Err(err) => {
                let msg = format!("{err:#}");
                assert!(msg.contains("unknown framework var"), "got: {msg}");
            }
        }
        Ok(())
    }

    #[test]
    fn template_none_uses_base_defaults() -> Result<()> {
        let args = args_non_interactive(Some("none"), &[]);
        let selection = collect_framework_config(&args)?;
        assert!(selection.template.is_none());
        assert!(selection.config.contains_key("web_root") || selection.config.is_empty());
        Ok(())
    }

    #[test]
    fn template_omitted_uses_base_defaults() -> Result<()> {
        let args = args_non_interactive(None, &[]);
        let selection = collect_framework_config(&args)?;
        assert!(selection.template.is_none());
        Ok(())
    }

    #[test]
    fn non_interactive_database_services_are_validated() -> Result<()> {
        let mut args = args_non_interactive(None, &[]);
        args.services = vec![String::from("postgres"), String::from("valkey")];
        assert_eq!(collect_database_services(&args)?, args.services);
        args.services = vec![String::from("unknown")];
        assert!(collect_database_services(&args).is_err());
        args.services = vec![String::from("postgres"), String::from("postgres")];
        assert!(collect_database_services(&args).is_err());
        Ok(())
    }
}
