use bonesdeploy_core::names;

use std::path::{Path, PathBuf};

use anyhow::Result;
use bonesdeploy_core::config::{Bones, validate_domain, validate_email};
use bonesdeploy_core::paths;
use rustinfra::operations::files::{DirectoryInput, LinkInput, TemplateInput};
use rustinfra::operations::server::{Script, ScriptInput, ScriptTemplate, ScriptTemplateInput};
use rustinfra::operations::systemd::ServiceInput;
use rustinfra::operations::{DeployError, DeployHost, PreparedDeploy};
use serde_json::{Map, Value, json};

use super::operations::{directory, escaped, link, named, service, shell, template};
use crate::{
    ProvisioningContext, ProvisioningReport, ProvisioningRuntime, TemplateResolver, reject_custom_provisioning,
};

struct SslAssets {
    default_certificate: PathBuf,
    default_server: PathBuf,
    router: PathBuf,
    nginx_validation: PathBuf,
    etckeeper_record: PathBuf,
}

#[derive(Clone, Copy)]
struct RouterSpec<'a> {
    template_data: &'a Map<String, Value>,
    ssl_enabled: bool,
    stage: &'a str,
}

/// Prepares the complete HTTPS provisioning plan without connecting.
///
/// # Errors
/// Returns an error for invalid domain/email values, missing templates, or an
/// invalid operation graph.
pub fn prepare_ssl(
    context: &ProvisioningContext,
    runtime: &ProvisioningRuntime,
    templates: &TemplateResolver,
) -> Result<PreparedDeploy> {
    validate_domain(&context.domain)?;
    validate_email(&context.email)?;
    let assets = SslAssets {
        default_certificate: templates.resolve(names::DEFAULT_CERTIFICATE_TEMPLATE)?,
        default_server: templates.resolve(names::DEFAULT_DENY_TEMPLATE)?,
        router: templates.resolve(names::ROUTER_TEMPLATE)?,
        nginx_validation: templates.resolve(names::NGINX_VALIDATION_SCRIPT)?,
        etckeeper_record: templates.resolve(names::ETCKEEPER_TEMPLATE)?,
    };
    let template_data = context.template_data()?;
    runtime.prepare(|host| {
        let mut acme = DirectoryInput::new(&context.paths.acme_webroot);
        acme.mode = Some(0o755);
        directory(host, "Ensure ACME webroot exists", acme)?;
        add_default_deny(host, context, &assets, &template_data)?;
        add_router(
            host,
            context,
            &assets,
            RouterSpec { template_data: &template_data, ssl_enabled: false, stage: "certbot challenge" },
        )?;
        add_certificate(host, context)?;
        add_router(
            host,
            context,
            &assets,
            RouterSpec { template_data: &template_data, ssl_enabled: true, stage: "SSL enable" },
        )?;
        add_etckeeper(host, &assets.etckeeper_record)?;
        Ok(())
    })
}

/// Configures HTTPS directly through RustInfra.
///
/// # Errors
/// Returns an error for existing custom provisioning, invalid configuration,
/// missing templates, or failed operations.
pub async fn apply_ssl(config: &Bones, project_root: &Path) -> Result<ProvisioningReport> {
    reject_custom_provisioning(project_root)?;
    let context = ProvisioningContext::from_bones(config)?;
    let runtime = ProvisioningRuntime::new(&context.server)?;
    let deploy = prepare_ssl(&context, &runtime, &TemplateResolver::new(project_root))?;
    runtime.execute(&deploy).await
}

fn add_default_deny(
    host: &mut DeployHost<'_>,
    context: &ProvisioningContext,
    assets: &SslAssets,
    template_data: &Map<String, Value>,
) -> Result<(), DeployError> {
    let data = Map::from_iter([
        ("cert".to_owned(), json!(context.paths.nginx_default_deny_ssl_certificate)),
        ("key".to_owned(), json!(context.paths.nginx_default_deny_ssl_certificate_key)),
    ]);
    host.add_stateful(
        ScriptTemplate,
        ScriptTemplateInput { source: assets.default_certificate.clone(), arguments: Vec::new(), data },
        named("Ensure nginx default-deny SSL certificate exists"),
    )?;

    let mut server = TemplateInput::new(&assets.default_server, &context.paths.nginx_default_deny_site_available);
    server.data = template_data.clone();
    template(host, "Deploy nginx default-deny server", server)?;
    link(
        host,
        "Enable nginx default-deny server",
        LinkInput::new(
            &context.paths.nginx_default_deny_site_enabled,
            &context.paths.nginx_default_deny_site_available,
        ),
    )?;
    let mut disable_default = LinkInput::new(&context.paths.nginx_default_site_enabled, "");
    disable_default.present = false;
    link(host, "Disable Debian default nginx site", disable_default)?;
    Ok(())
}

fn add_router(
    host: &mut DeployHost<'_>,
    context: &ProvisioningContext,
    assets: &SslAssets,
    spec: RouterSpec<'_>,
) -> Result<(), DeployError> {
    let mut data = spec.template_data.clone();
    data.insert(names::NGINX_SERVER_NAME.to_owned(), json!(context.domain));
    data.insert(names::NGINX_SSL_ENABLED.to_owned(), json!(spec.ssl_enabled));
    data.insert(names::NGINX_SSL_CERTIFICATE_PATH.to_owned(), json!(paths::ssl_certificate_path(&context.domain)));
    data.insert(
        names::NGINX_SSL_CERTIFICATE_KEY_PATH.to_owned(),
        json!(paths::ssl_certificate_key_path(&context.domain)),
    );
    let label = format!(" ({})", spec.stage);
    let mut router = TemplateInput::new(&assets.router, &context.paths.nginx_site_available);
    router.data = data;
    template(host, &format!("Deploy router nginx config{label}"), router)?;
    link(
        host,
        &format!("Enable router nginx site{label}"),
        LinkInput::new(&context.paths.nginx_site_enabled, &context.paths.nginx_site_available),
    )?;
    host.add_stateful(
        Script,
        ScriptInput { source: assets.nginx_validation.clone(), arguments: Vec::new() },
        named(&format!("Validate nginx configuration{label}")),
    )?;
    let mut service_input = ServiceInput::new("nginx");
    service_input.reloaded = true;
    service(host, &format!("Reload nginx{label}"), service_input)
}

fn add_certificate(host: &mut DeployHost<'_>, context: &ProvisioningContext) -> Result<(), DeployError> {
    let email = escaped(&context.email)?;
    let webroot = escaped(&context.paths.acme_webroot)?;
    let domain = escaped(&context.domain)?;
    let command = format!(
        "certbot certonly --non-interactive --agree-tos --email {email} --webroot -w {webroot} -d {domain} --keep-until-expiring"
    );
    shell(host, "Obtain or renew certificate", &command)
}

fn add_etckeeper(host: &mut DeployHost<'_>, template: &Path) -> Result<(), DeployError> {
    let mut data = Map::new();
    data.insert("message".to_owned(), json!("BonesInfra SSL provisioning"));
    host.add_stateful(
        ScriptTemplate,
        ScriptTemplateInput { source: template.to_path_buf(), arguments: Vec::new(), data },
        named(names::ETCKEEPER_RECORD_OPERATION),
    )?;
    Ok(())
}
