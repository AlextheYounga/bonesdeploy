use bonesdeploy_core::state;

use std::path::Path;

use rustinfra::operations::apt::{Packages, PackagesInput};
use rustinfra::operations::files::{File, FileInput};
use rustinfra::operations::server::{Script, ScriptInput};
use rustinfra::operations::{DeployError, DeployHost, OperationId};
use serde_json::{Map, Value, json};

use super::{
    directory, ensure_nginx_dirs, framework_template, log_directories, run_as_runtime, runtime_directory,
    shared_directories, start_application,
};
use crate::ProvisioningContext;
use crate::frameworks::{Language, Placeholder, ServerPlan};
use crate::provisioning::runtime::common::{escaped, named, require_service, shell, template};

#[derive(Clone, Copy)]
pub(super) struct Assets<'a> {
    pub(super) nginx: &'a Path,
    pub(super) apparmor: &'a Path,
    pub(super) service: &'a Path,
    pub(super) placeholder: Option<&'a Path>,
    pub(super) install_node: Option<&'a Path>,
}

pub(super) fn add(
    host: &mut DeployHost<'_>,
    context: &ProvisioningContext,
    data: &Map<String, Value>,
    plan: &ServerPlan,
    assets: Assets<'_>,
) -> Result<(), DeployError> {
    install_language(host, context, plan.language, assets.install_node)?;
    runtime_directory(host, context)?;
    log_directories(host, context)?;
    shared_directories(host, context, plan.shared_directories)?;

    let profile_name = format!("bonesdeploy-{}-{}", context.project_name, plan.name);
    let mut profile_data = data.clone();
    profile_data.extend([
        (state::APPARMOR_PROFILE_NAME.to_owned(), json!(profile_name)),
        ("apparmor_runtime".to_owned(), json!(plan.name)),
        ("apparmor_exec_paths".to_owned(), json!(plan.exec_paths)),
        ("apparmor_writable_paths".to_owned(), json!(plan.writable_paths)),
        ("apparmor_network".to_owned(), json!(plan.apparmor_network)),
    ]);
    let profile_path = context.paths.apparmor_profile(plan.name);
    template(
        host,
        &format!("Deploy {} AppArmor profile", plan.name),
        assets.apparmor,
        &profile_path,
        profile_data,
        Some(0o644),
    )?;
    shell(
        host,
        &format!("Load {} AppArmor profile", plan.name),
        &format!("apparmor_parser -r -T -W {}", escaped(&profile_path)),
    )?;
    shell(host, &format!("Enforce {} AppArmor profile", plan.name), &format!("aa-enforce {}", escaped(&profile_name)))?;
    seed_placeholder(host, context, data, plan, assets.placeholder)?;
    validate_placeholder(host, context, plan)?;

    let mut service_data = data.clone();
    service_data.extend([
        ("runtime_name".to_owned(), json!(plan.name)),
        ("runtime_label".to_owned(), json!(plan.label)),
        ("runtime_exec".to_owned(), json!(plan.command)),
        (state::APPARMOR_PROFILE_NAME.to_owned(), json!(profile_name)),
        ("runtime_write_paths".to_owned(), json!(plan.writable_paths.join(" "))),
        (
            "runtime_address_families".to_owned(),
            json!(if plan.tcp_port.is_some() { state::AF_UNIX_INET } else { state::AF_UNIX }),
        ),
    ]);
    template(
        host,
        &format!("Deploy {} systemd service", plan.name),
        assets.service,
        &context.paths.systemd_service(plan.name),
        service_data,
        Some(0o644),
    )?;
    require_service(host, context, plan.name)?;

    let proxy = plan.tcp_port.map_or_else(
        || format!("http://unix:{}/{}/{}.sock:", context.paths.runtime_socket_dir, plan.name, plan.name),
        state::tcp_proxy_target,
    );
    let mut nginx_data = data.clone();
    nginx_data.insert(state::APP_PROXY_TARGET.to_owned(), json!(proxy));
    framework_template(
        host,
        "Deploy per-site app nginx config",
        assets.nginx,
        &context.paths.site_nginx_config,
        nginx_data,
        context,
        0o640,
    )?;
    ensure_nginx_dirs(host, context)?;
    run_as_runtime(
        host,
        context,
        "Validate nginx configuration with app config",
        &format!("nginx -t -c {}", escaped(&context.paths.site_nginx_config)),
    )?;
    start_application(host, context, plan.name, &profile_name)
}

fn install_language(
    host: &mut DeployHost<'_>,
    context: &ProvisioningContext,
    language: Language,
    node_script: Option<&Path>,
) -> Result<(), DeployError> {
    match language {
        Language::Node => {
            let script = node_script
                .ok_or_else(|| DeployError::InconsistentOperation { id: OperationId::new("missing-node-installer") })?;
            host.add_stateful(
                Script,
                ScriptInput { source: script.to_path_buf(), arguments: vec![context.node_version.clone()] },
                named(&format!("Install Node.js v{}", context.node_version)),
            )?;
        }
        Language::Python => {
            let mut packages = PackagesInput::new([
                "build-essential",
                "ca-certificates",
                "libffi-dev",
                "libpq-dev",
                "libssl-dev",
                "pkg-config",
                "python3",
                "python3-dev",
                "python3-pip",
                "python3-venv",
            ]);
            packages.update = true;
            host.add_stateful(
                Packages,
                packages,
                named("Install distribution Python and native package build dependencies"),
            )?;
        }
        Language::Ruby => {
            let mut packages = PackagesInput::new([
                "build-essential",
                "default-libmysqlclient-dev",
                "git",
                "libpq-dev",
                "libsqlite3-dev",
                "pkg-config",
                "ruby",
                "ruby-bundler",
                "ruby-dev",
            ]);
            packages.update = true;
            host.add_stateful(
                Packages,
                packages,
                named("Install distribution Ruby and native gem build dependencies"),
            )?;
        }
    }
    Ok(())
}

fn seed_placeholder(
    host: &mut DeployHost<'_>,
    context: &ProvisioningContext,
    data: &Map<String, Value>,
    plan: &ServerPlan,
    source: Option<&Path>,
) -> Result<(), DeployError> {
    let source = source
        .ok_or_else(|| DeployError::InconsistentOperation { id: OperationId::new("missing-placeholder-template") })?;
    let root = &context.paths.placeholder_release;
    match &plan.placeholder {
        Placeholder::Django { module_path } => {
            shell(
                host,
                "Create placeholder venv with gunicorn",
                &format!("cd {} && /usr/bin/python3 -m venv .venv && .venv/bin/pip install gunicorn", escaped(root)),
            )?;
            let parent = Path::new(module_path).parent().and_then(Path::to_str).unwrap_or_default();
            directory(host, "Ensure placeholder config directory exists", &format!("{root}/{parent}"), context, 0o750)?;
            framework_template(
                host,
                "Seed placeholder WSGI application",
                source,
                &format!("{root}/{module_path}"),
                data.clone(),
                context,
                0o640,
            )?;
        }
        Placeholder::Next => seed_file(
            host,
            context,
            data,
            source,
            state::NEXT_STANDALONE_DIR,
            "server.js",
            "Next.js standalone server",
        )?,
        Placeholder::Nuxt => {
            seed_file(host, context, data, source, state::NUXT_SERVER_DIR, "index.mjs", "Nuxt nitro server")?;
        }
        Placeholder::Rails => {
            framework_template(
                host,
                "Seed placeholder Gemfile",
                source,
                &format!("{root}/Gemfile"),
                data.clone(),
                context,
                0o640,
            )?;
            shell(
                host,
                "Install placeholder gems",
                &format!(
                    "cd {} && BUNDLE_DISABLE_VERSION_CHECK=true BUNDLE_PATH=vendor/bundle BUNDLE_VERSION=system /usr/bin/bundle install",
                    escaped(root)
                ),
            )?;
            let rack = source.with_file_name("placeholder-config.ru.j2");
            framework_template(
                host,
                "Seed placeholder Rack config",
                &rack,
                &format!("{root}/config.ru"),
                data.clone(),
                context,
                0o640,
            )?;
        }
        Placeholder::SvelteKit => {
            seed_file(host, context, data, source, "build", "index.js", "SvelteKit build entrypoint")?;
            let mut file = FileInput::new(format!("{root}/.env"));
            file.user = Some("root".to_owned());
            file.group = Some(context.runtime_group.clone());
            file.mode = Some(0o640);
            host.add_stateful(File, file, named("Seed blank .env for SvelteKit placeholder"))?;
        }
    }
    Ok(())
}

fn validate_placeholder(
    host: &mut DeployHost<'_>,
    context: &ProvisioningContext,
    plan: &ServerPlan,
) -> Result<(), DeployError> {
    let Some(command) = plan.validation_command.as_deref() else {
        return Ok(());
    };
    run_as_runtime(host, context, "Validate application output as runtime user", command)
}

fn seed_file(
    host: &mut DeployHost<'_>,
    context: &ProvisioningContext,
    data: &Map<String, Value>,
    source: &Path,
    directory_path: &str,
    file: &str,
    label: &str,
) -> Result<(), DeployError> {
    let root = format!("{}/{directory_path}", context.paths.placeholder_release);
    directory(host, &format!("Ensure placeholder {directory_path} directory exists"), &root, context, 0o750)?;
    framework_template(
        host,
        &format!("Seed placeholder {label}"),
        source,
        &format!("{root}/{file}"),
        data.clone(),
        context,
        0o750,
    )
}
