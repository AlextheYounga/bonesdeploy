use bonesdeploy_core::state;

use std::borrow::Cow;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use bonesdeploy_core::config::{Bones, RuntimeBackend};
use crc32fast::Hasher;
use rustinfra::operations::apt::{Packages, PackagesInput};
use rustinfra::operations::files::{Directory, DirectoryInput, Download, DownloadInput, Template, TemplateInput};
use rustinfra::operations::server::{Script, ScriptInput, ScriptTemplate, ScriptTemplateInput, Shell, ShellCommands};
use rustinfra::operations::systemd::{DaemonReload, DaemonReloadInput, Service, ServiceInput};
use rustinfra::operations::{DeployError, DeployHost, OperationOptions, PreparedDeploy};
use serde_json::{Map, json};

use crate::{
    ProvisioningContext, ProvisioningReport, ProvisioningRuntime, TemplateResolver, reject_custom_provisioning,
};

const PORT_MIN: u32 = 20_000;
const PORT_RANGE: u32 = 40_000;

struct TunnelAssets {
    package_source: PathBuf,
    nginx_route: PathBuf,
    enable_nginx_site: PathBuf,
    service: PathBuf,
    nginx_validation: PathBuf,
    etckeeper_record: PathBuf,
}

/// Returns the stable unprivileged loopback port used by a project's tunnel.
#[must_use]
pub fn tunnel_port(project_name: &str) -> u16 {
    let mut hasher = Hasher::new();
    hasher.update(project_name.as_bytes());
    u16::try_from(PORT_MIN + hasher.finalize() % PORT_RANGE).unwrap_or(20_000)
}

/// Prepares Quick Tunnel installation and startup without connecting.
///
/// # Errors
/// Returns an error for an unsupported Docker configuration, missing templates,
/// or an invalid operation graph.
pub fn prepare_tunnel_start(
    context: &ProvisioningContext,
    runtime: &ProvisioningRuntime,
    templates: &TemplateResolver,
) -> Result<PreparedDeploy> {
    validate_supported(context)?;
    let assets = resolve_assets(templates)?;
    let port = tunnel_port(&context.project_name);
    let template_data = context.template_data()?;
    runtime.prepare(|host| {
        add_install(host, &assets)?;
        let route_data =
            Map::from_iter([("cloudflared_port".to_owned(), json!(port)), ("paths".to_owned(), json!(context.paths))]);
        let mut route = TemplateInput::new(&assets.nginx_route, &context.paths.nginx_cloudflared_site_available);
        route.data = route_data;
        host.add_stateful(Template, route, named("Deploy Cloudflared nginx loopback route"))?;
        host.add_stateful(
            Script,
            ScriptInput {
                source: assets.enable_nginx_site.clone(),
                arguments: vec![
                    context.paths.nginx_cloudflared_site_available.clone(),
                    context.paths.nginx_cloudflared_site_enabled.clone(),
                ],
            },
            named("Enable and validate Cloudflared nginx loopback route"),
        )?;
        reload_nginx(host, "Reload nginx for Cloudflared loopback route")?;

        let mut service_data = template_data.clone();
        service_data.insert("cloudflared_port".to_owned(), json!(port));
        let mut service = TemplateInput::new(&assets.service, &context.paths.systemd_cloudflared_service);
        service.data = service_data;
        service.user = Some("root".to_owned());
        service.group = Some("root".to_owned());
        service.mode = Some(0o644);
        host.add_stateful(Template, service, named("Deploy project quick tunnel service"))?;

        let requirement = state::joined(
            &context.paths.systemd_site_target_requires,
            &state::service_unit(&context.project_name, "cloudflared"),
        );
        shell(host, "Keep quick tunnel outside the site target", &format!("rm -f -- {}", escaped(&requirement)))?;
        host.add_with_options(
            DaemonReload,
            DaemonReloadInput::default(),
            named("Reload systemd after quick tunnel change"),
        )?;
        let mut tunnel = ServiceInput::new(state::service_unit(&context.project_name, "cloudflared"));
        tunnel.enabled = Some(true);
        tunnel.restarted = true;
        host.add_stateful(Service, tunnel, named("Enable and start project quick tunnel"))?;
        add_etckeeper(host, &assets.etckeeper_record, "BonesInfra Quick Tunnel start")?;
        Ok(())
    })
}

/// Prepares complete Quick Tunnel removal without connecting.
///
/// # Errors
/// Returns an error for missing validation assets or an invalid operation graph.
pub fn prepare_tunnel_stop(
    context: &ProvisioningContext,
    runtime: &ProvisioningRuntime,
    templates: &TemplateResolver,
) -> Result<PreparedDeploy> {
    let assets = resolve_assets(templates)?;
    runtime.prepare(|host| {
        let service = state::service_unit(&context.project_name, "cloudflared");
        let requirement = state::joined(&context.paths.systemd_site_target_requires, &service);
        shell(
            host,
            "Stop and remove project quick tunnel",
            &format!(
                "if systemctl cat -- {service} >/dev/null 2>&1; then systemctl disable --now -- {service}; fi; rm -f -- {} {} {} {}",
                escaped(&context.paths.systemd_cloudflared_service),
                escaped(&context.paths.nginx_cloudflared_site_enabled),
                escaped(&context.paths.nginx_cloudflared_site_available),
                escaped(&requirement)
            ),
        )?;
        host.add_with_options(
            DaemonReload,
            DaemonReloadInput::default(),
            named("Reload systemd after quick tunnel removal"),
        )?;
        host.add_stateful(
            Script,
            ScriptInput { source: assets.nginx_validation.clone(), arguments: Vec::new() },
            named("Validate nginx configuration after Cloudflared removal"),
        )?;
        reload_nginx(host, "Reload nginx after Cloudflared removal")?;
        add_etckeeper(host, &assets.etckeeper_record, "BonesInfra Quick Tunnel stop")?;
        Ok(())
    })
}

/// Installs and starts a project Quick Tunnel directly through RustInfra.
///
/// # Errors
/// Returns an error for unsupported custom provisioning, invalid configuration,
/// missing templates, or failed operations.
pub async fn provision_tunnel_start(config: &Bones, project_root: &Path) -> Result<ProvisioningReport> {
    run(config, project_root, prepare_tunnel_start).await
}

/// Stops and removes a project Quick Tunnel directly through RustInfra.
///
/// # Errors
/// Returns an error for unsupported custom provisioning, invalid configuration,
/// missing templates, or failed operations.
pub async fn provision_tunnel_stop(config: &Bones, project_root: &Path) -> Result<ProvisioningReport> {
    run(config, project_root, prepare_tunnel_stop).await
}

async fn run(
    config: &Bones,
    project_root: &Path,
    prepare: fn(&ProvisioningContext, &ProvisioningRuntime, &TemplateResolver) -> Result<PreparedDeploy>,
) -> Result<ProvisioningReport> {
    reject_custom_provisioning(project_root)?;
    let context = ProvisioningContext::from_bones(config)?;
    let runtime = ProvisioningRuntime::new(&context.server)?;
    let deploy = prepare(&context, &runtime, &TemplateResolver::new(project_root))?;
    runtime.execute(&deploy).await
}

fn resolve_assets(templates: &TemplateResolver) -> Result<TunnelAssets> {
    Ok(TunnelAssets {
        package_source: templates.resolve("shared/apt/cloudflared.list.j2")?,
        nginx_route: templates.resolve("shared/nginx/cloudflared.conf.j2")?,
        enable_nginx_site: templates.resolve("shared/scripts/enable-nginx-site.sh")?,
        service: templates.resolve("shared/systemd/cloudflared.service.j2")?,
        nginx_validation: templates.resolve(state::NGINX_VALIDATION_SCRIPT)?,
        etckeeper_record: templates.resolve(state::ETCKEEPER_TEMPLATE)?,
    })
}

fn validate_supported(context: &ProvisioningContext) -> Result<()> {
    if context.backend == RuntimeBackend::Docker && context.compose_port.is_none() {
        bail!("Quick Tunnel requires Docker Compose managed nginx ingress (compose_port)");
    }
    Ok(())
}

fn add_install(host: &mut DeployHost<'_>, assets: &TunnelAssets) -> Result<(), DeployError> {
    let mut keyring = DirectoryInput::new(state::APT_KEYRINGS);
    keyring.user = Some("root".to_owned());
    keyring.group = Some("root".to_owned());
    keyring.mode = Some(0o755);
    host.add_stateful(Directory, keyring, named("Ensure APT keyring directory exists"))?;

    let mut key =
        DownloadInput::new("https://pkg.cloudflare.com/cloudflare-main.gpg", "/etc/apt/keyrings/cloudflare-main.gpg");
    key.user = Some("root".to_owned());
    key.group = Some("root".to_owned());
    key.mode = Some(0o644);
    host.add_stateful(Download, key, named("Install Cloudflare package signing key"))?;

    let mut source = TemplateInput::new(&assets.package_source, "/etc/apt/sources.list.d/cloudflared.list");
    source.user = Some("root".to_owned());
    source.group = Some("root".to_owned());
    source.mode = Some(0o644);
    host.add_stateful(Template, source, named("Install Cloudflare package source"))?;

    let mut package = PackagesInput::new(["cloudflared"]);
    package.update = true;
    host.add_stateful(Packages, package, named("Install cloudflared"))?;
    Ok(())
}

fn reload_nginx(host: &mut DeployHost<'_>, name: &str) -> Result<(), DeployError> {
    let mut nginx = ServiceInput::new("nginx");
    nginx.reloaded = true;
    host.add_stateful(Service, nginx, named(name))?;
    Ok(())
}

fn add_etckeeper(host: &mut DeployHost<'_>, template: &Path, message: &str) -> Result<(), DeployError> {
    let mut data = Map::new();
    data.insert("message".to_owned(), json!(message));
    host.add_stateful(
        ScriptTemplate,
        ScriptTemplateInput { source: template.to_path_buf(), arguments: Vec::new(), data },
        named(state::ETCKEEPER_RECORD_OPERATION),
    )?;
    Ok(())
}

fn shell(host: &mut DeployHost<'_>, name: &str, command: &str) -> Result<(), DeployError> {
    host.add_with_options(Shell, ShellCommands::from(command), named(name))?;
    Ok(())
}

fn escaped(value: &str) -> String {
    shell_escape::escape(Cow::Borrowed(value)).into_owned()
}

fn named(name: &str) -> OperationOptions {
    OperationOptions { name: Some(name.to_owned()), ..OperationOptions::default() }
}
