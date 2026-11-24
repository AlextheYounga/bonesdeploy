#![allow(
    clippy::too_many_arguments,
    reason = "framework operation registrars keep runtime paths, assets, and permissions explicit"
)]

mod laravel;
mod server;

use bonesdeploy_core::state;

use std::path::{Path, PathBuf};

use anyhow::Result;
use bonesdeploy_core::paths;
use rustinfra::operations::files::{Directory, DirectoryInput, Template, TemplateInput};
use rustinfra::operations::systemd::{Service, ServiceInput};
use rustinfra::operations::{DeployError, DeployHost, OperationId};
use serde_json::{Map, Value, json};

use super::common::{escaped, named, shell};
use crate::frameworks::{Language, Placeholder, RuntimePlan};
use crate::{ProvisioningContext, TemplateResolver};

pub(super) enum Assets {
    Custom,
    Static {
        nginx: PathBuf,
        placeholder: PathBuf,
    },
    Server {
        nginx: PathBuf,
        apparmor: PathBuf,
        service: PathBuf,
        placeholder: Option<PathBuf>,
        install_node: Option<PathBuf>,
    },
    Laravel {
        nginx: PathBuf,
        queue: PathBuf,
        php_pool: PathBuf,
        cleanup_pools: PathBuf,
    },
}

impl Assets {
    pub(super) fn resolve(templates: &TemplateResolver, plan: &RuntimePlan) -> Result<Self> {
        Ok(match plan {
            RuntimePlan::Custom => Self::Custom,
            RuntimePlan::Static { nginx_template, placeholder_template, .. } => Self::Static {
                nginx: templates.resolve(nginx_template)?,
                placeholder: templates.resolve(placeholder_template)?,
            },
            RuntimePlan::Server(server) => Self::Server {
                nginx: templates.resolve(server.nginx_template)?,
                apparmor: templates.resolve(server.apparmor_template)?,
                service: templates.resolve("shared/systemd/app.service.j2")?,
                placeholder: placeholder_template(templates, &server.placeholder)?,
                install_node: (server.language == Language::Node)
                    .then(|| templates.resolve("shared/scripts/install-node.sh"))
                    .transpose()?,
            },
            RuntimePlan::Laravel => Self::Laravel {
                nginx: templates.resolve("frameworks/laravel/nginx/laravel-site-nginx.conf.j2")?,
                queue: templates.resolve("frameworks/laravel/queue-worker.service.j2")?,
                php_pool: templates.resolve("shared/php/php-fpm-pool.conf.j2")?,
                cleanup_pools: templates.resolve("shared/scripts/cleanup-orphaned-php-pools.sh.j2")?,
            },
        })
    }
}

pub(super) fn add(
    host: &mut DeployHost<'_>,
    context: &ProvisioningContext,
    template_data: &Map<String, Value>,
    plan: &RuntimePlan,
    assets: &Assets,
) -> Result<(), DeployError> {
    match (plan, assets) {
        (RuntimePlan::Custom, Assets::Custom) => Ok(()),
        (RuntimePlan::Static { root, .. }, Assets::Static { nginx, placeholder }) => {
            add_static(host, context, template_data, root, nginx, placeholder)
        }
        (RuntimePlan::Server(plan), Assets::Server { nginx, apparmor, service, placeholder, install_node }) => {
            server::add(
                host,
                context,
                template_data,
                plan,
                server::Assets {
                    nginx,
                    apparmor,
                    service,
                    placeholder: placeholder.as_deref(),
                    install_node: install_node.as_deref(),
                },
            )
        }
        (RuntimePlan::Laravel, Assets::Laravel { nginx, queue, php_pool, cleanup_pools }) => {
            laravel::add(host, context, template_data, nginx, queue, php_pool, cleanup_pools)
        }
        _ => Err(DeployError::InconsistentOperation { id: OperationId::new("framework-plan-assets-mismatch") }),
    }
}

fn add_static(
    host: &mut DeployHost<'_>,
    context: &ProvisioningContext,
    data: &Map<String, Value>,
    root: &str,
    nginx: &Path,
    placeholder: &Path,
) -> Result<(), DeployError> {
    runtime_directory(host, context)?;
    let placeholder_root = state::joined(&context.paths.placeholder_release, root);
    directory(host, "Ensure static placeholder directory exists", &placeholder_root, context, 0o750)?;
    framework_template(
        host,
        "Seed static placeholder index page",
        placeholder,
        &format!("{placeholder_root}/index.html"),
        data.clone(),
        context,
        0o640,
    )?;
    let mut nginx_data = data.clone();
    nginx_data.insert("static_root".to_owned(), json!(root));
    framework_template(
        host,
        "Deploy per-site static nginx config",
        nginx,
        &context.paths.site_nginx_config,
        nginx_data,
        context,
        0o640,
    )?;
    ensure_nginx_dirs(host, context)?;
    run_as_runtime(
        host,
        context,
        "Validate nginx configuration with static config",
        &format!("nginx -t -c {}", escaped(&context.paths.site_nginx_config)),
    )
}

fn start_application(
    host: &mut DeployHost<'_>,
    context: &ProvisioningContext,
    name: &str,
    profile: &str,
) -> Result<(), DeployError> {
    shell(
        host,
        &format!("Remove {name} service from direct multi-user boot"),
        &format!(
            "rm -f -- /etc/systemd/system/multi-user.target.wants/{}",
            state::service_unit(&context.project_name, name)
        ),
    )?;
    let unit = state::service_unit(&context.project_name, name);
    let mut service = ServiceInput::new(&unit);
    service.daemon_reload = true;
    host.add_stateful(Service, service, named(&format!("Start {name} service")))?;
    if !profile.is_empty() {
        shell(
            host,
            &format!("Verify {unit} attached to {profile}"),
            &format!(
                "attempt=0; while [ \"$attempt\" -lt 20 ]; do pid=$(systemctl show -p MainPID --value {}); if [ \"$pid\" != 0 ] && grep -qF -- {} /proc/$pid/attr/current; then exit 0; fi; attempt=$((attempt + 1)); sleep 0.1; done; false",
                escaped(&unit),
                escaped(profile)
            ),
        )?;
    }
    Ok(())
}

fn runtime_directory(host: &mut DeployHost<'_>, context: &ProvisioningContext) -> Result<(), DeployError> {
    directory(host, "Ensure runtime socket directory exists", &context.paths.runtime_socket_dir, context, 0o711)
}

fn log_directories(host: &mut DeployHost<'_>, context: &ProvisioningContext) -> Result<(), DeployError> {
    let mut root = DirectoryInput::new(paths::BONESDEPLOY_LOG_ROOT);
    root.user = Some("root".to_owned());
    root.group = Some("root".to_owned());
    root.mode = Some(0o755);
    host.add_stateful(Directory, root, named("Ensure BonesDeploy log root exists"))?;
    directory(host, "Ensure per-project log directory exists", &context.paths.site_log_dir, context, 0o750)
}

fn shared_directories(
    host: &mut DeployHost<'_>,
    context: &ProvisioningContext,
    paths: &[&str],
) -> Result<(), DeployError> {
    for path in paths {
        directory(
            host,
            &format!("Ensure shared directory {path}"),
            &state::joined(&context.paths.shared, path),
            context,
            0o770,
        )?;
    }
    Ok(())
}

fn ensure_nginx_dirs(host: &mut DeployHost<'_>, context: &ProvisioningContext) -> Result<(), DeployError> {
    directory(
        host,
        "Ensure runtime socket directory exists before nginx validation",
        &context.paths.runtime_socket_dir,
        context,
        0o711,
    )?;
    directory(
        host,
        "Ensure nginx runtime directory exists before nginx validation",
        &context.paths.runtime_nginx_dir,
        context,
        0o711,
    )
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

fn framework_template(
    host: &mut DeployHost<'_>,
    name: &str,
    source: &Path,
    destination: &str,
    data: Map<String, Value>,
    context: &ProvisioningContext,
    mode: u16,
) -> Result<(), DeployError> {
    let mut input = TemplateInput::new(source, destination);
    input.data = data;
    input.user = Some("root".to_owned());
    input.group = Some(context.runtime_group.clone());
    input.mode = Some(mode);
    host.add_stateful(Template, input, named(name))?;
    Ok(())
}

fn run_as_runtime(
    host: &mut DeployHost<'_>,
    context: &ProvisioningContext,
    name: &str,
    command: &str,
) -> Result<(), DeployError> {
    shell(
        host,
        name,
        &format!(
            "runuser -u {} -- sh -lc {}",
            escaped(&context.runtime_user),
            escaped(&format!(
                "cd {} && HOME=$(getent passwd {} | cut -d: -f6) XDG_CONFIG_HOME=$HOME/.config {command}",
                context.paths.current, context.runtime_user
            ))
        ),
    )
}

fn placeholder_template(templates: &TemplateResolver, placeholder: &Placeholder) -> Result<Option<PathBuf>> {
    Ok(Some(match placeholder {
        Placeholder::Django { .. } => templates.resolve("frameworks/django/django/placeholder-wsgi.py.j2")?,
        Placeholder::Next => templates.resolve("frameworks/next/next/placeholder-server.js.j2")?,
        Placeholder::Nuxt => templates.resolve("frameworks/nuxt/nuxt/placeholder-server.mjs.j2")?,
        Placeholder::Rails => templates.resolve("frameworks/rails/rails/placeholder-Gemfile.j2")?,
        Placeholder::SvelteKit => templates.resolve("frameworks/sveltekit/sveltekit/placeholder-index.js.j2")?,
    }))
}
