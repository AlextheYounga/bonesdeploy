use bonesdeploy_core::state;

use anyhow::{Result, bail};

use super::{Language, Placeholder, RuntimePlan, ServerPlan, string_value};
use crate::ProvisioningContext;

pub(super) fn plan(context: &ProvisioningContext) -> Result<RuntimePlan> {
    let module = string_value(context, state::WSGI_MODULE, "config.wsgi:application")?;
    let Some((import, callable)) = module.split_once(':') else {
        bail!("wsgi_module must use the form package.module:callable");
    };
    if !identifier(callable) || !import.split('.').all(identifier) {
        bail!("wsgi_module must use the form package.module:callable");
    }
    let module_path = format!("{}.py", import.replace('.', "/"));
    let gunicorn = format!("{}/.venv/bin/gunicorn", context.paths.current);
    Ok(RuntimePlan::Server(ServerPlan {
        name: "gunicorn",
        label: "Gunicorn",
        language: Language::Python,
        nginx_template: "frameworks/django/nginx/app-site-nginx.conf.j2",
        apparmor_template: "frameworks/django/app-profile.j2",
        placeholder: Placeholder::Django { module_path },
        command: format!(
            "{gunicorn} {} --bind unix:{}/gunicorn/gunicorn.sock --worker-tmp-dir {}/gunicorn",
            shell_escape::escape(module.as_str().into()),
            context.paths.runtime_socket_dir,
            context.paths.runtime_socket_dir,
        ),
        validation_command: Some(format!(
            "{}/.venv/bin/gunicorn --check-config {}",
            context.paths.current,
            shell_escape::escape(module.as_str().into())
        )),
        exec_paths: vec![gunicorn],
        writable_paths: vec![format!("{}/media", context.paths.shared)],
        shared_directories: &["media"],
        tcp_port: None,
        apparmor_network: "network unix stream,",
    }))
}

fn identifier(value: &str) -> bool {
    let mut chars = value.chars();
    chars.next().is_some_and(|ch| ch == '_' || ch.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}
