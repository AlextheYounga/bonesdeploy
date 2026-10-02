use std::fs;

use anyhow::{Result, bail};
use bonesdeploy::config::{Bones, bootstrap_ssh_user, load, write_local_environment};
use bonesdeploy::frameworks::Framework;
use bonesdeploy_core::config::{Runtime, RuntimeBackend};
use serde_json::{Map, Value, json};

fn sample_config(project_name: &str) -> Bones {
    let mut config = Bones::default();
    config.project_name = String::from(project_name);
    config.host = String::from("deploy.example.com");
    config.port = String::from("22");
    config.branch = String::from("master");
    config
}

fn bones_with_runtime(template: &str, extra: Map<String, Value>) -> Result<Bones> {
    let mut config = Bones::default();
    config.project_name = String::from("atlas");
    let mut runtime_vars = Map::new();
    runtime_vars.insert("template".to_string(), Value::String(template.to_string()));
    runtime_vars.insert("web_root".to_string(), Value::String("public".to_string()));
    runtime_vars.extend(extra);
    config.runtime = serde_json::from_value(json!(runtime_vars))?;
    Ok(config)
}

#[test]
fn bootstrap_ssh_user_resolves_defaults_config_and_whitespace() {
    let mut config = Bones::default();
    config.ssh_user = String::new();
    assert_eq!(bootstrap_ssh_user(&config), "root");
    config.ssh_user = String::from("ubuntu");
    assert_eq!(bootstrap_ssh_user(&config), "ubuntu");
    config.ssh_user = String::from("   ");
    assert_eq!(bootstrap_ssh_user(&config), "root");
    config.ssh_user = String::from("  ubuntu  ");
    assert_eq!(bootstrap_ssh_user(&config), "ubuntu");
}

#[test]
fn write_local_environment_round_trips_dotenv_values() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let path = temp.path().join("save.env");
    let mut config = sample_config("phoenix");
    config.domain = String::from("app.example.com");
    config.email = String::from("ops@example.com");
    config.runtime.template = String::from("next");
    config.runtime.backend = RuntimeBackend::Docker;
    config.runtime.web_root = String::from("dist");
    config.runtime.compose_port = Some(8080);
    config.runtime.compose_wait_timeout = 240;

    write_local_environment(&config, &path)?;
    let content = fs::read_to_string(&path)?;
    assert!(!content.contains("SSL_ENABLED"));
    assert!(content.contains("DOMAIN=app.example.com"));
    assert!(content.contains("EMAIL=ops@example.com"));
    let loaded = load(&path)?;
    assert_eq!(loaded.project_name, "phoenix");
    assert_eq!(loaded.ssh_user, "root");
    assert_eq!(loaded.host, "deploy.example.com");
    assert_eq!(loaded.port, "22");
    assert_eq!(loaded.branch, "master");
    assert_eq!(loaded.runtime.template, "next");
    assert_eq!(loaded.runtime.backend, RuntimeBackend::Docker);
    assert_eq!(loaded.runtime.web_root, "dist");
    assert_eq!(loaded.runtime.compose_port, Some(8080));
    assert_eq!(loaded.runtime.compose_wait_timeout, 240);
    Ok(())
}

#[test]
fn write_local_environment_writes_flat_local_input_file() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let path = temp.path().join("flat.env");
    write_local_environment(&sample_config("phoenix"), &path)?;
    assert!(fs::read_to_string(path)?.lines().all(|line| !line.starts_with('[')));
    Ok(())
}

#[test]
fn framework_wire_values_parse_and_display() -> Result<()> {
    for wire in ["angular", "django", "laravel", "next", "nuxt", "rails", "sveltekit", "vue", "custom"] {
        assert_eq!(Framework::parse(wire)?.to_string(), wire);
    }
    assert!(Framework::parse("unknown").is_err());
    assert_eq!(Framework::Next.display_name(), "Next.js");
    assert_eq!(Framework::SvelteKit.display_name(), "SvelteKit");
    assert_eq!(Framework::Angular.display_name(), "Angular");
    assert_eq!(Framework::Django.display_name(), "Django");
    Ok(())
}

#[test]
fn custom_is_the_empty_framework_fallback() -> Result<()> {
    assert!(Framework::Custom.questions().is_empty());
    assert!(Framework::Custom.validate_answers(&Map::new()).is_ok());
    assert!(Framework::Custom.environment_example("atlas", "").is_none());
    assert!(Framework::Custom.build_environment_example(&Runtime::default()).is_none());
    assert!(Framework::Custom.runtime_defaults()?.is_none());
    Ok(())
}

#[test]
fn next_and_nuxt_default_to_node_25_without_changing_other_frameworks() -> Result<()> {
    for framework in [Framework::Next, Framework::Nuxt] {
        let defaults = framework.runtime_defaults()?.ok_or_else(|| anyhow::anyhow!("missing defaults"))?;
        assert_eq!(defaults.get("node_version"), Some(&Value::String("25.9.0".into())));
    }

    let defaults = Framework::SvelteKit.runtime_defaults()?.ok_or_else(|| anyhow::anyhow!("missing defaults"))?;
    assert_eq!(defaults.get("node_version"), Some(&Value::String("24.19.0".into())));
    Ok(())
}

#[test]
fn angular_defaults_to_the_static_browser_artifact() -> Result<()> {
    assert!(Framework::Angular.questions().is_empty());
    let defaults = Framework::Angular.runtime_defaults()?.ok_or_else(|| anyhow::anyhow!("missing defaults"))?;

    assert_eq!(defaults.get("template"), Some(&Value::String("angular".into())));
    assert_eq!(defaults.get("web_root"), Some(&Value::String("dist/browser".into())));
    assert_eq!(
        defaults.get("permissions"),
        Some(&json!([
            {"path": "*", "type": "dir", "mode": 750, "recursive": false},
            {"path": "*", "type": "file", "mode": 640},
            {"path": "dist", "type": "dir", "mode": 755, "recursive": true}
        ]))
    );
    Ok(())
}

#[test]
fn environment_examples_use_project_name_in_shared_paths() -> Result<()> {
    let laravel = Framework::Laravel
        .environment_example("atlas", "example.com")
        .ok_or_else(|| anyhow::anyhow!("missing Laravel environment defaults"))?;
    assert!(laravel.contains("/srv/sites/atlas/shared/storage"));
    assert!(laravel.contains("APP_URL=https://example.com"));
    assert!(!laravel.contains("<project>"));
    let django = Framework::Django
        .environment_example("atlas", "")
        .ok_or_else(|| anyhow::anyhow!("missing Django environment defaults"))?;
    assert!(django.contains("/srv/sites/atlas/shared/database.sqlite"));
    let rails = Framework::Rails
        .environment_example("atlas", "")
        .ok_or_else(|| anyhow::anyhow!("missing Rails environment defaults"))?;
    assert!(rails.contains("/srv/sites/atlas/shared/storage/production.sqlite3"));
    Ok(())
}

#[test]
fn build_environments_use_selected_language_versions() -> Result<()> {
    for (framework, key, version, expected, old) in [
        (Framework::Laravel, "php_version", "8.3", "PHP_VERSION=8.3", "PHP_VERSION=8.5"),
        (Framework::Rails, "ruby_version", "3.4.8", "RUBY_VERSION=3.4.8", "RUBY_VERSION=3.3.8"),
    ] {
        let runtime: Runtime = serde_json::from_value(Value::Object(
            [(key.to_string(), Value::String(version.to_string()))].into_iter().collect(),
        ))?;
        let environment = framework
            .build_environment_example(&runtime)
            .ok_or_else(|| anyhow::anyhow!("missing build environment"))?;
        assert!(environment.contains(expected));
        assert!(!environment.contains(old));
    }
    Ok(())
}

#[test]
fn django_does_not_expose_an_unused_python_version() -> Result<()> {
    let defaults = Framework::Django.runtime_defaults()?.ok_or_else(|| anyhow::anyhow!("missing Django defaults"))?;
    let environment = Framework::Django
        .build_environment_example(&Runtime::default())
        .ok_or_else(|| anyhow::anyhow!("missing Django build environment"))?;

    assert!(!defaults.contains_key("python_version"));
    assert!(Framework::Django.questions().iter().all(|question| question.key != "python_version"));
    assert!(!environment.contains("PYTHON_VERSION"));
    Ok(())
}

#[test]
fn rails_requires_supported_exact_ruby_versions_for_new_projects() -> Result<()> {
    let mut answers = Map::new();
    for question in Framework::Rails.questions() {
        answers.insert(question.key.to_string(), question.default_value());
    }
    answers.insert("ruby_version".to_string(), Value::String("3.4.8".to_string()));
    Framework::Rails.validate_answers(&answers)?;

    answers.insert("ruby_version".to_string(), Value::String("3.4".to_string()));
    assert!(Framework::Rails.validate_answers(&answers).is_err_and(|error| error.to_string().contains("not one of")));
    Ok(())
}

#[test]
fn framework_answer_validation_preserves_all_cases() -> Result<()> {
    let mut answers = Map::new();
    answers.insert("php_verison".to_string(), Value::String("8.5".to_string()));
    let error = Framework::Laravel.validate_answers(&answers).err().ok_or_else(|| anyhow::anyhow!("expected error"))?;
    assert!(error.to_string().contains("unknown framework var"));
    assert!(error.to_string().contains("php_verison"));

    answers.clear();
    answers.insert("php_version".to_string(), Value::String("8.6".to_string()));
    assert!(Framework::Laravel.validate_answers(&answers).is_err_and(|error| error.to_string().contains("not one of")));
    answers.insert("php_version".to_string(), Value::Bool(true));
    assert!(Framework::Laravel.validate_answers(&answers).is_err_and(|error| error.to_string().contains("wrong type")));

    answers.clear();
    for question in Framework::Laravel.questions() {
        answers.insert(question.key.to_string(), question.default_value());
    }
    Framework::Laravel.validate_answers(&answers).map_err(|error| anyhow::anyhow!("defaults failed: {error}"))?;
    Ok(())
}

#[test]
fn static_framework_configuration_overrides_only_static_web_roots() -> Result<()> {
    let mut next = bones_with_runtime("next", [("is_static".to_string(), Value::Bool(true))].into_iter().collect())?;
    Framework::Next.configure(&mut next);
    assert_eq!(next.runtime.web_root, "out");
    let mut nuxt = bones_with_runtime("nuxt", [("is_static".to_string(), Value::Bool(true))].into_iter().collect())?;
    Framework::Nuxt.configure(&mut nuxt);
    assert_eq!(nuxt.runtime.web_root, ".output/public");
    let mut server = bones_with_runtime("next", [("is_static".to_string(), Value::Bool(false))].into_iter().collect())?;
    Framework::Next.configure(&mut server);
    if server.runtime.web_root != "public" {
        bail!("server web root changed")
    }
    Ok(())
}
