use bonesdeploy_core::state;

use std::io::Error;
use std::path::{Path, PathBuf};

use anyhow::Result;
use rustinfra::connectors::Connector;
use rustinfra::facts::server::LinuxDistribution;
use rustinfra::facts::{FactExecutor, FactOptions};
use rustinfra::operations::apt::{Packages, PackagesInput};
use rustinfra::operations::files::{Directory, DirectoryInput, Download, DownloadInput, Template, TemplateInput};
use rustinfra::operations::server::{Shell, ShellCommands};
use rustinfra::operations::systemd::{DaemonReload, DaemonReloadInput, Service, ServiceInput};
use rustinfra::operations::{DeployError, DeployHost, StatefulOperation, StatefulOperationFuture};
use serde_json::{Map, Value, json};

use super::common::{self, named, require_service};
use crate::{ProvisioningContext, TemplateResolver};

pub(super) struct Assets {
    service: PathBuf,
    nginx: PathBuf,
    docker_source: PathBuf,
    common: common::Assets,
}

impl Assets {
    pub(super) fn resolve(templates: &TemplateResolver) -> Result<Self> {
        Ok(Self {
            service: templates.resolve("shared/systemd/compose.service.j2")?,
            nginx: templates.resolve("shared/nginx/compose-site-nginx.conf.j2")?,
            docker_source: templates.resolve("shared/apt/docker.sources.j2")?,
            common: common::Assets::resolve(templates)?,
        })
    }
}

#[derive(Clone, Debug)]
struct DockerRepositoryInput {
    source: PathBuf,
}

#[derive(Clone, Copy, Debug, Default)]
struct DockerRepository;

impl StatefulOperation for DockerRepository {
    type Input = DockerRepositoryInput;

    fn name(&self) -> &'static str {
        "bonesinfra.docker_repository"
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
            let info = FactExecutor::new(connector, fact_options).get(&LinuxDistribution, &()).await?;
            let distribution = info
                .release_meta
                .get("ID")
                .filter(|value| matches!(value.as_str(), "debian" | "ubuntu"))
                .ok_or_else(|| Error::other("Docker repository requires Debian or Ubuntu"))?;
            let codename = info
                .release_meta
                .get("CODENAME")
                .filter(|value| !value.is_empty())
                .ok_or_else(|| Error::other("host distribution codename is unavailable"))?;

            let mut key = DownloadInput::new(
                format!("https://download.docker.com/linux/{distribution}/gpg"),
                "/etc/apt/keyrings/docker.asc",
            );
            key.user = Some("root".to_owned());
            key.group = Some("root".to_owned());
            key.mode = Some(0o644);
            let mut commands = Download.commands(&key, connector, fact_options).await?;

            let mut source = TemplateInput::new(&input.source, "/etc/apt/sources.list.d/docker.sources");
            source.data = Map::from_iter([
                ("distribution".to_owned(), json!(distribution)),
                ("codename".to_owned(), json!(codename)),
            ]);
            source.user = Some("root".to_owned());
            source.group = Some("root".to_owned());
            source.mode = Some(0o644);
            commands.extend(Template.commands(&source, connector, fact_options).await?);
            Ok(commands)
        })
    }
}

pub(super) fn add(
    host: &mut DeployHost<'_>,
    context: &ProvisioningContext,
    data: &Map<String, Value>,
    assets: &Assets,
) -> Result<(), DeployError> {
    remove_stale_runtime(host, context)?;
    install_docker(host, &assets.docker_source)?;
    if context.compose_port.is_some() {
        common::setup(host, context, data, &assets.common, true)?;
    } else {
        common::template(
            host,
            "Deploy site systemd target",
            &assets.common.site_target,
            &context.paths.systemd_site_target,
            data.clone(),
            Some(0o644),
        )?;
    }
    common::template(
        host,
        "Deploy Compose runtime service",
        &assets.service,
        &context.paths.systemd_service("compose"),
        data.clone(),
        Some(0o644),
    )?;
    require_service(host, context, "compose")?;
    if let Some(port) = context.compose_port {
        let mut nginx_data = data.clone();
        nginx_data.insert(state::APP_PROXY_TARGET.to_owned(), json!(state::tcp_proxy_target(port)));
        let mut nginx = TemplateInput::new(&assets.nginx, &context.paths.site_nginx_config);
        nginx.data = nginx_data;
        nginx.user = Some("root".to_owned());
        nginx.group = Some(context.runtime_group.clone());
        nginx.mode = Some(0o640);
        host.add_stateful(Template, nginx, named("Deploy per-site app nginx config"))?;
    }
    let mut service = ServiceInput::new(state::service_unit(&context.project_name, "compose"));
    service.daemon_reload = true;
    host.add_stateful(Service, service, named("Start compose service"))?;
    if context.compose_port.is_some() {
        common::finish(host, context, &assets.common)?;
    }
    Ok(())
}

fn install_docker(host: &mut DeployHost<'_>, docker_source: &Path) -> Result<(), DeployError> {
    let mut prerequisites = PackagesInput::new(["ca-certificates", "curl"]);
    prerequisites.update = true;
    host.add_stateful(Packages, prerequisites, named("Install Docker repository prerequisites"))?;
    let mut keyring = DirectoryInput::new(state::APT_KEYRINGS);
    keyring.user = Some("root".to_owned());
    keyring.group = Some("root".to_owned());
    keyring.mode = Some(0o755);
    host.add_stateful(Directory, keyring, named("Ensure APT keyring directory exists"))?;
    host.add_stateful(
        DockerRepository,
        DockerRepositoryInput { source: docker_source.to_path_buf() },
        named("Install Docker package repository"),
    )?;
    let mut conflicts = PackagesInput::new(["docker.io", "docker-compose", "podman-docker", "containerd", "runc"]);
    conflicts.present = false;
    host.add_stateful(Packages, conflicts, named("Remove packages that conflict with Docker Engine"))?;
    let mut docker = PackagesInput::new([
        "docker-ce",
        "docker-ce-cli",
        "containerd.io",
        "docker-buildx-plugin",
        "docker-compose-plugin",
    ]);
    docker.update = true;
    host.add_stateful(Packages, docker, named("Install Docker Engine and Compose plugin"))?;
    let mut service = ServiceInput::new("docker");
    service.enabled = Some(true);
    host.add_stateful(Service, service, named("Enable Docker Engine"))?;
    shell(host, "Verify Docker Compose plugin", "docker compose version >/dev/null")
}

fn remove_stale_runtime(host: &mut DeployHost<'_>, context: &ProvisioningContext) -> Result<(), DeployError> {
    let names =
        ["docker", "nginx", "worker", "django", "gunicorn", "next", "nuxt", "puma", "rails", "sveltekit", "vue"];
    let units = names.iter().map(|name| state::service_unit(&context.project_name, name)).collect::<Vec<_>>().join(" ");
    shell(
        host,
        "Stop stale managed native runtime services",
        &format!("systemctl disable --now {units} 2>/dev/null || true"),
    )?;
    let files = names
        .iter()
        .flat_map(|name| [context.paths.systemd_service(name), context.paths.systemd_service_requirement(name)])
        .collect::<Vec<_>>()
        .join(" ");
    shell(
        host,
        "Remove stale managed native runtime files",
        &format!(
            "rm -f -- {files} {} {} {}",
            context.paths.site_nginx_config, context.paths.nginx_site_available, context.paths.nginx_site_enabled
        ),
    )?;
    shell(
        host,
        "Remove stale Laravel Docker artifacts",
        &format!(
            "rm -rf -- /var/lib/bonesdeploy/runtime-images/{}; rm -f -- /etc/php/*/fpm/pool.d/{}.conf /run/{}/php-fpm.sock",
            context.project_name, context.project_name, context.project_name
        ),
    )?;
    host.add_with_options(
        DaemonReload,
        DaemonReloadInput::default(),
        named("Reload systemd after stale runtime removal"),
    )?;
    let mut nginx = ServiceInput::new("nginx");
    nginx.running = None;
    nginx.reloaded = true;
    host.add_stateful(Service, nginx, named("Reload nginx after stale ingress removal"))?;
    Ok(())
}

fn shell(host: &mut DeployHost<'_>, name: &str, command: &str) -> Result<(), DeployError> {
    host.add_with_options(Shell, ShellCommands::from(command), named(name))?;
    Ok(())
}
