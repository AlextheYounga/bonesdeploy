use bonesdeploy_core::state;

use anyhow::Result;

use super::{Language, Placeholder, RuntimePlan, ServerPlan, numeric_version};
use crate::ProvisioningContext;

pub(super) fn plan(context: &ProvisioningContext) -> Result<RuntimePlan> {
    numeric_version(&context.node_version, 3, state::NODE_VERSION)?;
    let node = state::node_binary(&context.node_version);
    let socket = format!("{}/sveltekit/sveltekit.sock", context.paths.runtime_socket_dir);
    let origin =
        if context.domain.is_empty() { "https://localhost".to_owned() } else { format!("https://{}", context.domain) };
    Ok(RuntimePlan::Server(ServerPlan {
        name: "sveltekit",
        label: "SvelteKit app server",
        language: Language::Node,
        nginx_template: "frameworks/sveltekit/nginx/app-site-nginx.conf.j2",
        apparmor_template: "frameworks/sveltekit/app-profile.j2",
        placeholder: Placeholder::SvelteKit,
        command: format!(
            "/usr/bin/env --chdir={} NODE_ENV=production SOCKET_PATH={socket} ORIGIN={origin} {node} --env-file=.env build",
            context.paths.current,
        ),
        validation_command: Some("test -e build".to_owned()),
        exec_paths: vec![node],
        writable_paths: Vec::new(),
        shared_directories: &[],
        tcp_port: None,
        apparmor_network: "network unix stream,",
    }))
}
