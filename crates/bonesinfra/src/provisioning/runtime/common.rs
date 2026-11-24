#![allow(
    clippy::too_many_arguments,
    reason = "provisioning registrars keep security-sensitive ownership, modes, and destinations explicit"
)]

use bonesdeploy_core::state;

use std::borrow::Cow;
use std::path::{Path, PathBuf};

use anyhow::Result;
use bonesdeploy_core::paths;
use rustinfra::connectors::Connector;
use rustinfra::facts::files::{Link as LinkFact, PathInfo, PathInput};
use rustinfra::facts::{FactExecutor, FactOptions};
use rustinfra::operations::files::{Directory, DirectoryInput, Link, LinkInput, Template, TemplateInput};
use rustinfra::operations::server::{Script, ScriptInput, ScriptTemplate, ScriptTemplateInput, Shell, ShellCommands};
use rustinfra::operations::systemd::{DaemonReload, DaemonReloadInput, Service, ServiceInput};
use rustinfra::operations::{DeployError, DeployHost, OperationOptions, StatefulOperation, StatefulOperationFuture};
use serde_json::{Map, Value, json};

use crate::{ProvisioningContext, TemplateResolver};

pub(super) struct Assets {
    nginx_profile: PathBuf,
    pub(super) site_target: PathBuf,
    nginx_service: PathBuf,
    default_certificate: PathBuf,
    default_site: PathBuf,
    router: PathBuf,
    validate_nginx: PathBuf,
    etckeeper: PathBuf,
}

#[derive(Clone, Debug)]
struct CertificateAwareRouterInput {
    template: TemplateInput,
    certificate: String,
    key: String,
}

#[derive(Clone, Copy, Debug, Default)]
struct CertificateAwareRouter;

impl StatefulOperation for CertificateAwareRouter {
    type Input = CertificateAwareRouterInput;

    fn name(&self) -> &'static str {
        "bonesinfra.certificate_aware_router"
    }

    fn is_idempotent(&self) -> bool {
        true
    }

    fn commands<'a>(
        &'a self,
        input: &'a Self::Input,
        connector: &'a dyn Connector,
        fact_options: &'a FactOptions,
    ) -> StatefulOperationFuture<'a> {
        Box::pin(async move {
            let facts = FactExecutor::new(connector, fact_options);
            let certificate = facts.get(&LinkFact, &PathInput::from(input.certificate.clone())).await?;
            let key = facts.get(&LinkFact, &PathInput::from(input.key.clone())).await?;
            let mut template = input.template.clone();
            template.data.insert(
                state::NGINX_SSL_ENABLED.to_owned(),
                json!(matches!(certificate, PathInfo::Present(_)) && matches!(key, PathInfo::Present(_))),
            );
            Template.commands(&template, connector, fact_options).await
        })
    }
}

impl Assets {
    pub(super) fn resolve(templates: &TemplateResolver) -> Result<Self> {
        Ok(Self {
            nginx_profile: templates.resolve("shared/apparmor/project-nginx-profile.j2")?,
            site_target: templates.resolve("shared/systemd/site.target.j2")?,
            nginx_service: templates.resolve("shared/nginx/site-nginx.service.j2")?,
            default_certificate: templates.resolve(state::DEFAULT_CERTIFICATE_TEMPLATE)?,
            default_site: templates.resolve(state::DEFAULT_DENY_TEMPLATE)?,
            router: templates.resolve(state::ROUTER_TEMPLATE)?,
            validate_nginx: templates.resolve(state::NGINX_VALIDATION_SCRIPT)?,
            etckeeper: templates.resolve(state::ETCKEEPER_TEMPLATE)?,
        })
    }
}

pub(super) fn setup(
    host: &mut DeployHost<'_>,
    context: &ProvisioningContext,
    template_data: &Map<String, Value>,
    assets: &Assets,
    uses_tcp: bool,
) -> Result<(), DeployError> {
    let mut data = template_data.clone();
    data.insert(
        "nginx_apparmor_network".to_owned(),
        json!(if uses_tcp { "network inet stream," } else { "network unix stream," }),
    );
    data.insert(state::APPARMOR_PROFILE_NAME.to_owned(), json!(format!("bonesdeploy-{}-nginx", context.project_name)));
    shell(host, "Verify apparmor kernel enabled", &format!("cat {}", escaped(&context.paths.apparmor_enabled_param)))?;
    template(
        host,
        "Deploy per-project nginx AppArmor profile",
        &assets.nginx_profile,
        &context.paths.nginx_apparmor_profile,
        data.clone(),
        Some(0o644),
    )?;
    shell(
        host,
        "Load updated nginx AppArmor profile",
        &format!("apparmor_parser -r {}", escaped(&context.paths.nginx_apparmor_profile)),
    )?;
    shell(
        host,
        "Ensure nginx profile is in enforce mode",
        &format!("aa-enforce {}", escaped(&context.paths.nginx_apparmor_profile)),
    )?;

    template(
        host,
        "Deploy site systemd target",
        &assets.site_target,
        &context.paths.systemd_site_target,
        data.clone(),
        Some(0o644),
    )?;
    shell(
        host,
        "Reconcile site systemd target memberships",
        &format!(
            "rm -rf -- {0}; install -d -o root -g root -m 0755 -- {0}",
            escaped(&context.paths.systemd_site_target_requires)
        ),
    )?;
    directory(host, "Ensure socket directory exists", &context.paths.runtime_socket_dir, context, 0o711)?;
    directory(host, "Ensure nginx runtime directory exists", &context.paths.runtime_nginx_dir, context, 0o711)?;
    directory(host, "Ensure conf directory exists", &context.paths.conf_root, context, 0o750)?;
    data.insert(
        "nginx_address_families".to_owned(),
        json!(if uses_tcp { state::AF_UNIX_INET } else { state::AF_UNIX }),
    );
    data.insert("nginx_ip_loopback_only".to_owned(), json!(uses_tcp));
    template(
        host,
        "Deploy per-site nginx systemd service",
        &assets.nginx_service,
        &context.paths.systemd_site_nginx_service,
        data.clone(),
        Some(0o644),
    )?;
    require_service(host, context, "nginx")?;
    host.add_with_options(
        DaemonReload,
        DaemonReloadInput::default(),
        named("Reload systemd after site-nginx service change"),
    )?;

    let cert_data = Map::from_iter([
        ("cert".to_owned(), json!(context.paths.nginx_default_deny_ssl_certificate)),
        ("key".to_owned(), json!(context.paths.nginx_default_deny_ssl_certificate_key)),
    ]);
    host.add_stateful(
        ScriptTemplate,
        ScriptTemplateInput { source: assets.default_certificate.clone(), arguments: Vec::new(), data: cert_data },
        named("Ensure nginx default-deny SSL certificate exists"),
    )?;
    template(
        host,
        "Deploy nginx default-deny server",
        &assets.default_site,
        &context.paths.nginx_default_deny_site_available,
        template_data.clone(),
        Some(0o644),
    )?;
    link(
        host,
        "Enable nginx default-deny server",
        &context.paths.nginx_default_deny_site_enabled,
        &context.paths.nginx_default_deny_site_available,
        true,
    )?;
    link(host, "Disable Debian default nginx site", &context.paths.nginx_default_site_enabled, "", false)?;

    if !context.domain.is_empty() {
        let mut router = template_data.clone();
        let certificate = paths::ssl_certificate_path(&context.domain);
        let key = paths::ssl_certificate_key_path(&context.domain);
        router.extend([
            (state::NGINX_SERVER_NAME.to_owned(), json!(context.domain)),
            (state::NGINX_SSL_CERTIFICATE_PATH.to_owned(), json!(certificate)),
            (state::NGINX_SSL_CERTIFICATE_KEY_PATH.to_owned(), json!(key)),
        ]);
        let mut template = TemplateInput::new(&assets.router, &context.paths.nginx_site_available);
        template.data = router;
        template.user = Some("root".to_owned());
        template.group = Some("root".to_owned());
        template.mode = Some(0o644);
        host.add_stateful(
            CertificateAwareRouter,
            CertificateAwareRouterInput { template, certificate, key },
            named("Deploy router nginx config"),
        )?;
        link(
            host,
            "Enable router nginx site",
            &context.paths.nginx_site_enabled,
            &context.paths.nginx_site_available,
            true,
        )?;
        script(host, "Validate nginx configuration", &assets.validate_nginx)?;
    }
    Ok(())
}

pub(super) fn finish(
    host: &mut DeployHost<'_>,
    context: &ProvisioningContext,
    assets: &Assets,
) -> Result<(), DeployError> {
    if context.domain.is_empty() {
        link(host, "Disable project nginx router", &context.paths.nginx_site_enabled, "", false)?;
        shell(
            host,
            "Remove project nginx router config",
            &format!("rm -f -- {}", escaped(&context.paths.nginx_site_available)),
        )?;
        script(host, "Validate nginx configuration after router removal", &assets.validate_nginx)?;
        reload(host, "Reload nginx after router removal", "nginx")?;
    }
    let mut nginx = ServiceInput::new("nginx");
    nginx.enabled = Some(true);
    host.add_stateful(Service, nginx, named("Ensure nginx service is enabled and started"))?;
    shell(
        host,
        "Remove nginx service from direct multi-user boot",
        &format!("rm -f -- /etc/systemd/system/multi-user.target.wants/{}-nginx.service", context.project_name),
    )?;
    let mut target = ServiceInput::new(format!("{}.target", context.project_name));
    target.enabled = Some(true);
    target.restarted = true;
    target.daemon_reload = true;
    host.add_stateful(Service, target, named("Enable and restart site systemd target"))?;
    shell(host, "Reload nginx to apply site config changes", "systemctl reload nginx")?;
    let mut data = Map::new();
    data.insert("message".to_owned(), json!("BonesInfra runtime apply"));
    host.add_stateful(
        ScriptTemplate,
        ScriptTemplateInput { source: assets.etckeeper.clone(), arguments: Vec::new(), data },
        named(state::ETCKEEPER_RECORD_OPERATION),
    )?;
    Ok(())
}

pub(super) fn require_service(
    host: &mut DeployHost<'_>,
    context: &ProvisioningContext,
    service: &str,
) -> Result<(), DeployError> {
    let unit = format!("{}-{service}.service", context.project_name);
    shell(
        host,
        &format!("Require {unit} from site target"),
        &format!(
            "install -d -o root -g root -m 0755 -- {requires} && ln -sfn -- {source} {requires}/{unit}",
            requires = escaped(&context.paths.systemd_site_target_requires),
            source = escaped(&context.paths.systemd_service(service))
        ),
    )
}

pub(super) fn template(
    host: &mut DeployHost<'_>,
    name: &str,
    source: &Path,
    destination: &str,
    data: Map<String, Value>,
    mode: Option<u16>,
) -> Result<(), DeployError> {
    let mut input = TemplateInput::new(source, destination);
    input.data = data;
    input.user = Some("root".to_owned());
    input.group = Some("root".to_owned());
    input.mode = mode;
    host.add_stateful(Template, input, named(name))?;
    Ok(())
}

pub(super) fn shell(host: &mut DeployHost<'_>, name: &str, command: &str) -> Result<(), DeployError> {
    host.add_with_options(Shell, ShellCommands::from(command), named(name))?;
    Ok(())
}

pub(super) fn named(name: &str) -> OperationOptions {
    OperationOptions { name: Some(name.to_owned()), ..OperationOptions::default() }
}

pub(super) fn escaped(value: &str) -> String {
    shell_escape::escape(Cow::Borrowed(value)).into_owned()
}

fn directory(
    host: &mut DeployHost<'_>,
    name: &str,
    path: &str,
    context: &ProvisioningContext,
    mode: u16,
) -> Result<(), DeployError> {
    let mut input = DirectoryInput::new(path);
    input.user = Some(context.runtime_user.clone());
    input.group = Some(context.runtime_group.clone());
    input.mode = Some(mode);
    host.add_stateful(Directory, input, named(name))?;
    Ok(())
}

fn link(host: &mut DeployHost<'_>, name: &str, path: &str, target: &str, present: bool) -> Result<(), DeployError> {
    let mut input = LinkInput::new(path, target);
    input.present = present;
    host.add_stateful(Link, input, named(name))?;
    Ok(())
}

fn script(host: &mut DeployHost<'_>, name: &str, source: &Path) -> Result<(), DeployError> {
    host.add_stateful(Script, ScriptInput { source: source.to_path_buf(), arguments: Vec::new() }, named(name))?;
    Ok(())
}

fn reload(host: &mut DeployHost<'_>, name: &str, service: &str) -> Result<(), DeployError> {
    let mut input = ServiceInput::new(service);
    input.running = None;
    input.reloaded = true;
    host.add_stateful(Service, input, named(name))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;
    use std::os::unix::fs::symlink;

    use rustinfra::connectors::{LocalConnector, RunOptions};
    use rustinfra::facts::FactOptions;
    use rustinfra::operations::{CommandExecutor, StatefulOperation};

    use super::{CertificateAwareRouter, CertificateAwareRouterInput};
    use rustinfra::operations::files::TemplateInput;

    #[tokio::test]
    async fn router_enables_tls_only_when_both_certificate_links_exist() -> Result<(), Box<dyn Error + Send + Sync>> {
        let directory = tempfile::tempdir()?;
        let source = directory.path().join("router.j2");
        let destination = directory.path().join("router.conf");
        let certificate = directory.path().join("fullchain.pem");
        let key = directory.path().join("privkey.pem");
        fs::write(&source, "{% if nginx_ssl_enabled %}tls{% else %}http{% endif %}\n")?;
        let input = CertificateAwareRouterInput {
            template: TemplateInput::new(&source, destination.to_string_lossy()),
            certificate: certificate.to_string_lossy().into_owned(),
            key: key.to_string_lossy().into_owned(),
        };
        let fact_options = FactOptions::default();

        let commands = CertificateAwareRouter.commands(&input, &LocalConnector, &fact_options).await?;
        CommandExecutor.execute(&LocalConnector, &commands, &RunOptions::default()).await?;
        assert_eq!(fs::read_to_string(&destination)?, "http");

        let target = directory.path().join("certificate-target");
        fs::write(&target, "certificate\n")?;
        symlink(&target, &certificate)?;
        symlink(&target, &key)?;
        let commands = CertificateAwareRouter.commands(&input, &LocalConnector, &fact_options).await?;
        CommandExecutor.execute(&LocalConnector, &commands, &RunOptions::default()).await?;
        assert_eq!(fs::read_to_string(destination)?, "tls");
        Ok(())
    }
}
