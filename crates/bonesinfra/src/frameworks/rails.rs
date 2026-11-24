use bonesdeploy_core::names;

use anyhow::{Result, bail};
use bonesdeploy_core::paths;

use super::{Language, Placeholder, RuntimePlan, ServerPlan, string_value};
use crate::ProvisioningContext;

pub(super) fn plan(context: &ProvisioningContext) -> Result<RuntimePlan> {
    let environment = string_value(context, names::RAILS_ENV, "production")?;
    if !environment.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')) {
        bail!("rails_env may contain only letters, digits, underscores, and hyphens");
    }
    let socket = format!("{}/puma/puma.sock", context.paths.runtime_socket_dir);
    let bundle = "/usr/bin/bundle";
    let bundle_command =
        format!("BUNDLE_DISABLE_VERSION_CHECK=true BUNDLE_PATH=vendor/bundle BUNDLE_VERSION=system {bundle}");
    Ok(RuntimePlan::Server(ServerPlan {
        name: "puma",
        label: "Puma",
        language: Language::Ruby,
        nginx_template: "frameworks/rails/nginx/app-site-nginx.conf.j2",
        apparmor_template: "frameworks/rails/app-profile.j2",
        placeholder: Placeholder::Rails,
        command: format!(
            "/usr/bin/env RAILS_ENV={environment} {bundle_command} exec puma -e {environment} -b unix://{socket}"
        ),
        validation_command: Some(format!("{bundle_command} exec puma --help >/dev/null")),
        exec_paths: vec![
            "/usr/bin/env".to_owned(),
            "/usr/bin/ruby*".to_owned(),
            bundle.to_owned(),
            format!("{}/\u{2a}/vendor/bundle/ruby/\u{2a}/bin/puma", context.paths.releases),
        ],
        writable_paths: ["tmp", "log", "storage"].map(|path| paths::joined(&context.paths.shared, path)).to_vec(),
        shared_directories: &["tmp", "log", "storage"],
        tcp_port: None,
        apparmor_network: "network unix stream,",
    }))
}
