use bonesdeploy_core::state;

use anyhow::Result;
use bonesdeploy_core::config::{Bones, RuntimeBackend};
use bonesinfra::provisioning::prepare_runtime;
use bonesinfra::{ProvisioningContext, ProvisioningRuntime, TemplateResolver};
use rustinfra::operations::PreparedDeploy;

fn names(deploy: &PreparedDeploy) -> Vec<String> {
    deploy
        .plan()
        .operations
        .iter()
        .filter_map(|operation| deploy.operation(&operation.id))
        .map(|operation| operation.display_name().to_owned())
        .collect()
}

fn config(framework: &str) -> Bones {
    let mut config = Bones::for_site("atlas");
    config.host = "server.example.com".to_owned();
    config.runtime.template = framework.to_owned();
    config
}

#[test]
fn built_in_frameworks_register_native_runtime_operations() -> Result<()> {
    for (framework, expected) in [
        ("angular", "Seed static placeholder index page"),
        ("vue", "Seed static placeholder index page"),
        ("django", "Deploy gunicorn systemd service"),
        ("laravel", "Deploy Laravel queue worker service"),
        ("rails", "Deploy puma systemd service"),
        ("sveltekit", "Deploy sveltekit systemd service"),
    ] {
        let project = tempfile::tempdir()?;
        let config = config(framework);
        bonesinfra::materialize_project_artifacts(project.path(), &config.runtime)?;
        let context = ProvisioningContext::from_bones(&config)?;
        let runtime = ProvisioningRuntime::new(&context.server)?;
        let operations = names(&prepare_runtime(&context, &runtime, &TemplateResolver::new(project.path()))?);
        assert!(operations.iter().any(|name| name == expected), "{framework} omitted {expected}");
        assert_eq!(operations.last().map(String::as_str), Some(state::ETCKEEPER_RECORD_OPERATION));
    }
    Ok(())
}

#[test]
fn next_and_nuxt_select_static_or_server_plans_from_typed_config() -> Result<()> {
    for framework in ["next", "nuxt"] {
        for is_static in [true, false] {
            let project = tempfile::tempdir()?;
            let mut config = config(framework);
            config.runtime.extra.insert(state::IS_STATIC.to_owned(), toml::Value::Boolean(is_static));
            bonesinfra::materialize_project_artifacts(project.path(), &config.runtime)?;
            let context = ProvisioningContext::from_bones(&config)?;
            let runtime = ProvisioningRuntime::new(&context.server)?;
            let operations = names(&prepare_runtime(&context, &runtime, &TemplateResolver::new(project.path()))?);
            assert_eq!(
                operations.iter().any(|name| name == "Seed static placeholder index page"),
                is_static,
                "{framework} selected the wrong runtime mode"
            );
        }
    }
    Ok(())
}

#[test]
fn compose_runtime_installs_docker_and_managed_ingress() -> Result<()> {
    let project = tempfile::tempdir()?;
    let mut config = config("");
    config.runtime.backend = RuntimeBackend::Docker;
    config.runtime.compose_port = Some(8080);
    bonesinfra::materialize_project_artifacts(project.path(), &config.runtime)?;
    let context = ProvisioningContext::from_bones(&config)?;
    let runtime = ProvisioningRuntime::new(&context.server)?;
    let operations = names(&prepare_runtime(&context, &runtime, &TemplateResolver::new(project.path()))?);

    assert!(operations.iter().any(|name| name == "Install Docker Engine and Compose plugin"));
    assert!(operations.iter().any(|name| name == "Deploy Compose runtime service"));
    assert!(operations.iter().any(|name| name == "Deploy per-site app nginx config"));
    Ok(())
}
