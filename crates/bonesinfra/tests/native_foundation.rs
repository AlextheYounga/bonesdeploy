use bonesdeploy_core::names;

use std::fs;

use anyhow::Result;
use bonesdeploy_core::config::{Bones, RuntimeBackend};
use bonesinfra::{ProvisioningContext, ProvisioningRuntime, TemplateResolver};
use rustinfra::operations::server::{Shell, ShellCommands};

fn config() -> Bones {
    let mut config = Bones::for_site("example");
    config.host = "server.example.com".to_owned();
    config.ssh_user = "root".to_owned();
    config.port = "2222".to_owned();
    config.domain = "example.com".to_owned();
    config.email = "ops@example.com".to_owned();
    config.runtime.template = "laravel".to_owned();
    config.runtime.backend = RuntimeBackend::Native;
    config.runtime.extra.insert(names::PHP_VERSION.to_owned(), toml::Value::String("8.4".to_owned()));
    config
}

#[test]
fn context_is_derived_directly_from_bones() -> Result<()> {
    let context = ProvisioningContext::from_bones(&config())?;

    assert_eq!(context.server.host, "server.example.com");
    assert_eq!(context.server.port, 2222);
    assert_eq!(context.paths.project_root, "/srv/sites/example");
    let template_data = context.template_data()?;
    assert_eq!(template_data[names::PHP_VERSION], "8.4");
    assert_eq!(template_data["paths"]["runtime_nginx_socket"], "/run/example/nginx/nginx.sock");
    Ok(())
}

#[test]
fn templates_resolve_only_from_the_project_snapshot() -> Result<()> {
    let project = tempfile::tempdir()?;
    let template = project.path().join("infra/templates/shared/nginx/site.conf.j2");
    fs::create_dir_all(template.parent().ok_or_else(|| anyhow::anyhow!("template has no parent"))?)?;
    fs::write(&template, "server {}\n")?;
    let resolver = TemplateResolver::new(project.path());

    assert_eq!(resolver.resolve("shared/nginx/site.conf.j2")?, template.canonicalize()?);
    assert!(resolver.resolve("../custom/runtime.py").is_err());
    assert!(resolver.resolve("shared/nginx/missing.j2").is_err());
    Ok(())
}

#[test]
fn context_rejects_non_scalar_framework_values() -> Result<()> {
    let mut config = config();
    config.runtime.extra.insert("bad".to_owned(), toml::Value::Array(Vec::new()));

    let Err(error) = ProvisioningContext::from_bones(&config) else {
        return Err(anyhow::anyhow!("arrays entered template data"));
    };

    assert!(error.to_string().contains("must be a scalar"));
    Ok(())
}

#[test]
fn runtime_builds_ssh_inventory_and_prepares_typed_operations() -> Result<()> {
    let context = ProvisioningContext::from_bones(&config())?;
    let runtime = ProvisioningRuntime::new(&context.server)?;
    let deploy = runtime.prepare(|host| {
        host.add(Shell, ShellCommands::from("true"))?;
        Ok(())
    })?;

    let host = runtime.inventory().get_host("server.example.com")?;
    let data = runtime.inventory().resolved_host_data(host.name())?;
    assert_eq!(data[names::SSH_USER], "root");
    assert_eq!(data[names::SSH_PORT], 2222);
    assert_eq!(deploy.plan().operations.len(), 1);
    Ok(())
}
