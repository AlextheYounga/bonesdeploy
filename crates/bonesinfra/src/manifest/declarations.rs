use bonesdeploy_core::state;

use std::collections::BTreeSet;
use std::path::Path;

use anyhow::Result;
use bonesdeploy_core::config::RuntimeBackend;
use bonesdeploy_core::paths;

use super::ArtifactKind;
use crate::ProvisioningContext;
use crate::frameworks::{self, Placeholder, RuntimePlan};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Artifact {
    pub name: String,
    pub path: String,
    pub kind: ArtifactKind,
    pub owner: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManagedService {
    pub name: String,
    pub unit: String,
    pub owner: String,
}

/// Returns built-in artifact and service declarations plus the runtime mode.
///
/// # Errors
/// Returns an error for unknown frameworks or invalid typed framework values.
pub fn declarations(context: &ProvisioningContext) -> Result<(Vec<Artifact>, Vec<ManagedService>, String)> {
    let mut artifacts = common_artifacts(context);
    let mut services = Vec::new();
    if context.backend == RuntimeBackend::Docker {
        artifacts.extend([
            artifact("Compose runtime service", context.paths.systemd_service("compose"), ArtifactKind::File, "docker"),
            artifact(
                "Compose runtime requirement",
                context.paths.systemd_service_requirement("compose"),
                ArtifactKind::Link,
                "docker",
            ),
        ]);
        services.push(service("Compose stack", state::service_unit(&context.project_name, "compose"), "docker"));
        add_nginx(context, &mut artifacts, &mut services);
        add_backup(context, &mut artifacts);
        add_ssl(context, &mut artifacts);
        deduplicate(&mut artifacts, &mut services);
        return Ok((artifacts, services, "compose".to_owned()));
    }

    add_nginx(context, &mut artifacts, &mut services);
    add_backup(context, &mut artifacts);
    let mode = add_framework(context, &mut artifacts, &mut services)?;
    add_ssl(context, &mut artifacts);
    deduplicate(&mut artifacts, &mut services);
    Ok((artifacts, services, mode))
}

fn common_artifacts(context: &ProvisioningContext) -> Vec<Artifact> {
    let p = &context.paths;
    vec![
        artifact("project root", &p.project_root, ArtifactKind::Directory, "setup"),
        artifact("releases directory", &p.releases, ArtifactKind::Directory, "setup"),
        artifact("shared directory", &p.shared, ArtifactKind::Directory, "setup"),
        artifact("current release link", &p.current, ArtifactKind::Link, "deploy"),
        artifact("placeholder release", &p.placeholder_release, ArtifactKind::Directory, "setup"),
        artifact("placeholder web root", &p.placeholder_web_root, ArtifactKind::Directory, "setup"),
        artifact("placeholder index", &p.placeholder_index, ArtifactKind::File, "setup"),
        artifact("project configuration directory", &p.conf_root, ArtifactKind::Directory, "runtime"),
        artifact("site systemd target", &p.systemd_site_target, ArtifactKind::File, "runtime"),
        artifact("site systemd requirements", &p.systemd_site_target_requires, ArtifactKind::Directory, "runtime"),
        artifact("runtime socket directory", &p.runtime_socket_dir, ArtifactKind::Directory, "runtime"),
    ]
}

fn add_nginx(context: &ProvisioningContext, artifacts: &mut Vec<Artifact>, services: &mut Vec<ManagedService>) {
    if context.backend == RuntimeBackend::Docker && context.compose_port.is_none() {
        return;
    }
    let p = &context.paths;
    artifacts.extend([
        artifact("nginx site configuration", &p.site_nginx_config, ArtifactKind::File, "runtime"),
        artifact("nginx site", &p.nginx_site_available, ArtifactKind::File, "runtime"),
        artifact("enabled nginx site", &p.nginx_site_enabled, ArtifactKind::Link, "runtime"),
        artifact("site nginx systemd service", &p.systemd_site_nginx_service, ArtifactKind::File, "runtime"),
        artifact("site nginx systemd requirement", &p.systemd_site_nginx_requirement, ArtifactKind::Link, "runtime"),
        artifact("nginx AppArmor profile", &p.nginx_apparmor_profile, ArtifactKind::File, "runtime"),
        artifact("runtime nginx directory", &p.runtime_nginx_dir, ArtifactKind::Directory, "runtime"),
        artifact("runtime nginx socket", &p.runtime_nginx_socket, ArtifactKind::Socket, "runtime"),
        artifact("runtime nginx PID", &p.runtime_nginx_pid, ArtifactKind::File, "runtime"),
    ]);
    services.push(service("site nginx", format!("{}-nginx.service", context.project_name), "runtime"));
}

fn add_backup(context: &ProvisioningContext, artifacts: &mut Vec<Artifact>) {
    if context.backup.is_configured() {
        artifacts.extend([
            artifact("Borg backup repository", &context.paths.backup_repository, ArtifactKind::Directory, "backup"),
            artifact("Borg passphrase file", &context.paths.backup_passphrase_file, ArtifactKind::File, "backup"),
            artifact("scheduled backup cron entry", &context.paths.backup_cron_file, ArtifactKind::File, "backup"),
        ]);
    }
}

fn add_ssl(context: &ProvisioningContext, artifacts: &mut Vec<Artifact>) {
    if !context.domain.is_empty() {
        artifacts.extend([
            artifact("ACME webroot", &context.paths.acme_webroot, ArtifactKind::Directory, "ssl"),
            artifact("ACME certificate", paths::ssl_certificate_path(&context.domain), ArtifactKind::Link, "ssl"),
            artifact(
                "ACME certificate key",
                paths::ssl_certificate_key_path(&context.domain),
                ArtifactKind::Link,
                "ssl",
            ),
        ]);
    }
}

fn add_framework(
    context: &ProvisioningContext,
    artifacts: &mut Vec<Artifact>,
    services: &mut Vec<ManagedService>,
) -> Result<String> {
    match frameworks::plan(context)? {
        RuntimePlan::Custom => Ok("custom".to_owned()),
        RuntimePlan::Static { root, .. } => {
            add_static(context, artifacts, root);
            Ok("static".to_owned())
        }
        RuntimePlan::Laravel => {
            add_laravel(context, artifacts, services);
            Ok("php".to_owned())
        }
        RuntimePlan::Server(server) => {
            add_server(context, artifacts, services, &server);
            Ok("server".to_owned())
        }
    }
}

fn add_static(context: &ProvisioningContext, artifacts: &mut Vec<Artifact>, root: &str) {
    let p = &context.paths;
    artifacts.extend([
        artifact(
            "static placeholder web root",
            state::joined(&p.placeholder_release, root),
            ArtifactKind::Directory,
            "framework",
        ),
        artifact(
            "static placeholder index",
            format!("{}/{root}/index.html", p.placeholder_release),
            ArtifactKind::File,
            "framework",
        ),
        artifact("current static web root", state::joined(&p.current, root), ArtifactKind::Directory, "framework"),
    ]);
}

fn add_server(
    context: &ProvisioningContext,
    artifacts: &mut Vec<Artifact>,
    services: &mut Vec<ManagedService>,
    server: &frameworks::ServerPlan,
) {
    let p = &context.paths;
    let runtime = server.name;
    artifacts.extend([
        artifact("application AppArmor profile", p.apparmor_profile(runtime), ArtifactKind::File, "framework"),
        artifact("application systemd service", p.systemd_service(runtime), ArtifactKind::File, "framework"),
        artifact(
            "application systemd requirement",
            p.systemd_service_requirement(runtime),
            ArtifactKind::Link,
            "framework",
        ),
        artifact("application runtime directory", p.runtime_service_dir(runtime), ArtifactKind::Directory, "framework"),
        artifact("application log directory", &p.site_log_dir, ArtifactKind::Directory, "framework"),
    ]);
    if server.tcp_port.is_none() {
        artifacts.push(artifact(
            "application runtime socket",
            p.runtime_service_socket(runtime),
            ArtifactKind::Socket,
            "framework",
        ));
    }
    artifacts.extend(placeholder_artifacts(&server.placeholder).into_iter().map(|(name, relative, kind)| {
        artifact(name, format!("{}/{relative}", p.placeholder_release), kind, "framework")
    }));
    services.push(service("application service", format!("{}-{runtime}.service", context.project_name), "framework"));
}

fn add_laravel(context: &ProvisioningContext, artifacts: &mut Vec<Artifact>, services: &mut Vec<ManagedService>) {
    let version = context.extra.get(state::PHP_VERSION).and_then(serde_json::Value::as_str).unwrap_or("8.5");
    artifacts.extend([
        artifact(
            "PHP-FPM pool configuration",
            state::php_pool(&context.project_name, version),
            ArtifactKind::File,
            "framework",
        ),
        artifact(
            "PHP-FPM socket",
            state::php_socket(&context.project_name, version),
            ArtifactKind::Socket,
            "framework",
        ),
        artifact("PHP log directory", &context.paths.site_log_dir, ArtifactKind::Directory, "framework"),
        artifact("current PHP web root", &context.paths.current_web_root, ArtifactKind::Directory, "framework"),
        artifact(
            "Laravel queue worker service",
            context.paths.systemd_service("worker"),
            ArtifactKind::File,
            "framework",
        ),
    ]);
    services.push(service("Laravel queue worker", format!("{}-worker.service", context.project_name), "framework"));
}

fn placeholder_artifacts(placeholder: &Placeholder) -> Vec<(&'static str, String, ArtifactKind)> {
    match placeholder {
        Placeholder::Django { module_path } => vec![
            ("Django placeholder virtual environment", ".venv".to_owned(), ArtifactKind::Directory),
            (
                "Django placeholder configuration",
                Path::new(module_path).parent().and_then(Path::to_str).unwrap_or_default().to_owned(),
                ArtifactKind::Directory,
            ),
            ("Django placeholder WSGI application", module_path.clone(), ArtifactKind::File),
        ],
        Placeholder::Next => vec![
            (
                "Next.js placeholder standalone directory",
                state::NEXT_STANDALONE_DIR.to_owned(),
                ArtifactKind::Directory,
            ),
            ("Next.js placeholder standalone server", ".next/standalone/server.js".to_owned(), ArtifactKind::File),
        ],
        Placeholder::Nuxt => vec![
            ("Nuxt placeholder server directory", state::NUXT_SERVER_DIR.to_owned(), ArtifactKind::Directory),
            ("Nuxt placeholder server", ".output/server/index.mjs".to_owned(), ArtifactKind::File),
        ],
        Placeholder::Rails => vec![
            ("Rails placeholder Gemfile", "Gemfile".to_owned(), ArtifactKind::File),
            ("Rails placeholder Rack configuration", "config.ru".to_owned(), ArtifactKind::File),
        ],
        Placeholder::SvelteKit => vec![
            ("SvelteKit placeholder build directory", "build".to_owned(), ArtifactKind::Directory),
            ("SvelteKit placeholder entrypoint", "build/index.js".to_owned(), ArtifactKind::File),
            ("SvelteKit placeholder environment", paths::DOT_ENV.to_owned(), ArtifactKind::File),
        ],
    }
}

fn artifact(name: &str, path: impl Into<String>, kind: ArtifactKind, owner: &str) -> Artifact {
    Artifact { name: name.to_owned(), path: path.into(), kind, owner: owner.to_owned() }
}

fn service(name: &str, unit: impl Into<String>, owner: &str) -> ManagedService {
    ManagedService { name: name.to_owned(), unit: unit.into(), owner: owner.to_owned() }
}

fn deduplicate(artifacts: &mut Vec<Artifact>, services: &mut Vec<ManagedService>) {
    let mut paths = BTreeSet::new();
    artifacts.retain(|artifact| paths.insert(artifact.path.clone()));
    let mut units = BTreeSet::new();
    services.retain(|service| units.insert(service.unit.clone()));
}
