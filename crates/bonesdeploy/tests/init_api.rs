use anyhow::{Result, bail};
use bonesdeploy::commands::init::Args;
use bonesdeploy::commands::init::config::collect_non_interactive;
use bonesdeploy::commands::init::framework::{
    collect_database_services, collect_framework_config, parse_framework_var,
};
use bonesdeploy::config::Bones;
use bonesdeploy_core::config::RuntimeBackend;
use bonesdeploy_core::paths;
use serde_json::Value;

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

fn incomplete_existing(project_name: &str) -> Bones {
    let mut config = Bones::default();
    config.remote_name = String::from("production");
    config.project_name = String::from(project_name);
    config.port = String::from("22");
    config.branch = String::from("main");
    config
}

#[test]
fn framework_var_parser_preserves_bool_string_and_error_cases() -> Result<()> {
    assert_eq!(parse_framework_var("is_static=true")?, (String::from("is_static"), Value::Bool(true)));
    assert_eq!(parse_framework_var("is_static=FALSE")?, (String::from("is_static"), Value::Bool(false)));
    assert_eq!(
        parse_framework_var("php_version=8.5")?,
        (String::from("php_version"), Value::String(String::from("8.5")))
    );
    assert!(parse_framework_var("is_static").is_err_and(|error| error.to_string().contains("KEY=VALUE")));
    assert!(parse_framework_var("=value").is_err_and(|error| error.to_string().contains("empty")));
    Ok(())
}

#[test]
fn framework_config_validates_vars_and_template_fallbacks() -> Result<()> {
    let selection = collect_framework_config(&args_non_interactive(Some("laravel"), &["php_version=8.5"]))?;
    assert_eq!(selection.template.as_deref(), Some("laravel"));
    assert_eq!(selection.config.get("php_version"), Some(&Value::String("8.5".to_string())));
    let error = collect_framework_config(&args_non_interactive(Some("laravel"), &["php_verison=8.5"]))
        .err()
        .ok_or_else(|| anyhow::anyhow!("unknown framework var should fail"))?;
    assert!(format!("{error:#}").contains("unknown framework var"));

    let none = collect_framework_config(&args_non_interactive(Some("none"), &[]))?;
    assert!(none.template.is_none());
    assert!(none.config.contains_key("web_root") || none.config.is_empty());
    let custom = collect_framework_config(&args_non_interactive(Some("custom"), &[]))?;
    assert_eq!(custom.template.as_deref(), Some("custom"));
    assert_eq!(custom.config.get("template"), Some(&Value::String("custom".into())));
    assert!(collect_framework_config(&args_non_interactive(None, &[]))?.template.is_none());
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

#[test]
fn non_interactive_config_uses_existing_and_cli_values() -> Result<()> {
    let existing = incomplete_existing("atlas");
    let mut args = args_non_interactive(None, &[]);
    args.project_name = None;
    let config = collect_non_interactive("workspace", Some(&existing), &args)?;
    assert_eq!(config.project_name, "atlas");
    assert_eq!(config.host, "deploy.example.com");
    assert_eq!(config.branch, "main");
    assert_eq!(config.remote_name, "production");
    assert_eq!(config.repo_path, paths::default_repo_path_for("atlas"));
    Ok(())
}

#[test]
fn non_interactive_config_records_and_validates_runtime_backend() -> Result<()> {
    let mut args = args_non_interactive(None, &[]);
    args.runtime_backend = Some(String::from("docker"));
    assert_eq!(collect_non_interactive("workspace", None, &args)?.runtime.backend, RuntimeBackend::Docker);
    args.runtime_backend = Some(String::from("compose"));
    assert!(collect_non_interactive("workspace", None, &args).is_err());
    Ok(())
}

#[test]
fn non_interactive_config_requires_host_when_not_inferred() -> Result<()> {
    let existing = incomplete_existing("atlas");
    let mut args = args_non_interactive(None, &[]);
    args.project_name = None;
    args.remote = Some(String::from("missing-test-remote"));
    args.host = None;
    let Err(error) = collect_non_interactive("workspace", Some(&existing), &args) else {
        bail!("missing host should fail");
    };
    assert!(error.to_string().contains("--host is required"));
    Ok(())
}
