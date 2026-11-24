use bonesdeploy_core::names;

use std::fs;
use std::path::Path;

use anyhow::Result;
use bonesdeploy_core::config::{Bones, RuntimeBackend};
use bonesinfra::provisioning::{prepare_tunnel_start, prepare_tunnel_stop};
use bonesinfra::{ProvisioningContext, ProvisioningRuntime, TemplateResolver};
use rustinfra::operations::PreparedDeploy;

fn write_assets(project: &Path) -> Result<()> {
    for path in [
        "shared/apt/cloudflared.list.j2",
        "shared/nginx/cloudflared.conf.j2",
        "shared/scripts/enable-nginx-site.sh",
        "shared/systemd/cloudflared.service.j2",
        names::NGINX_VALIDATION_SCRIPT,
        names::ETCKEEPER_TEMPLATE,
    ] {
        let target = project.join("infra/templates").join(path);
        fs::create_dir_all(target.parent().ok_or_else(|| anyhow::anyhow!("asset has no parent"))?)?;
        fs::write(target, "asset")?;
    }
    Ok(())
}

fn config() -> Bones {
    let mut config = Bones::for_site("atlas");
    config.host = "server.example.com".to_owned();
    config
}

fn names(deploy: &PreparedDeploy) -> Vec<String> {
    deploy
        .plan()
        .operations
        .iter()
        .filter_map(|operation| deploy.operation(&operation.id))
        .map(|operation| operation.display_name().to_owned())
        .collect()
}

#[test]
fn tunnel_plans_install_start_and_fully_remove_project_resources() -> Result<()> {
    let project = tempfile::tempdir()?;
    write_assets(project.path())?;
    let context = ProvisioningContext::from_bones(&config())?;
    let runtime = ProvisioningRuntime::new(&context.server)?;
    let resolver = TemplateResolver::new(project.path());

    let start = names(&prepare_tunnel_start(&context, &runtime, &resolver)?);
    assert!(start.iter().any(|name| name == "Install Cloudflare package signing key"));
    assert!(start.iter().any(|name| name == "Deploy project quick tunnel service"));
    assert!(start.iter().any(|name| name == "Enable and start project quick tunnel"));
    assert_eq!(start.last().map(String::as_str), Some(names::ETCKEEPER_RECORD_OPERATION));

    let stop = names(&prepare_tunnel_stop(&context, &runtime, &resolver)?);
    assert_eq!(stop.first().map(String::as_str), Some("Stop and remove project quick tunnel"));
    assert!(stop.iter().any(|name| name == "Validate nginx configuration after Cloudflared removal"));
    assert_eq!(stop.last().map(String::as_str), Some(names::ETCKEEPER_RECORD_OPERATION));
    Ok(())
}

#[test]
fn docker_tunnel_requires_managed_ingress_port() -> Result<()> {
    let project = tempfile::tempdir()?;
    write_assets(project.path())?;
    let mut config = config();
    config.runtime.backend = RuntimeBackend::Docker;
    config.runtime.compose_port = None;
    let context = ProvisioningContext::from_bones(&config)?;
    let runtime = ProvisioningRuntime::new(&context.server)?;

    assert!(prepare_tunnel_start(&context, &runtime, &TemplateResolver::new(project.path())).is_err());
    Ok(())
}
