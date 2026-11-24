use bonesdeploy_core::names;

use anyhow::Result;

use super::{Language, Placeholder, RuntimePlan, ServerPlan, bool_value, numeric_version};
use crate::ProvisioningContext;

pub(super) fn plan(context: &ProvisioningContext) -> Result<RuntimePlan> {
    numeric_version(&context.node_version, 3, names::NODE_VERSION)?;
    if bool_value(context, names::IS_STATIC, true)? {
        return Ok(RuntimePlan::Static {
            root: names::NUXT_PUBLIC_DIR,
            nginx_template: "frameworks/nuxt/nginx/static-site-nginx.conf.j2",
            placeholder_template: "frameworks/nuxt/nginx/index.html.j2",
        });
    }
    let node = names::node_binary(&context.node_version);
    let socket = format!("{}/nuxt/nuxt.sock", context.paths.runtime_socket_dir);
    Ok(RuntimePlan::Server(ServerPlan {
        name: "nuxt",
        label: "Nuxt app server",
        language: Language::Node,
        nginx_template: "frameworks/nuxt/nginx/app-site-nginx.conf.j2",
        apparmor_template: "frameworks/nuxt/app-profile.j2",
        placeholder: Placeholder::Nuxt,
        command: format!("/usr/bin/env NODE_ENV=production NITRO_UNIX_SOCKET={socket} {node} .output/server/index.mjs"),
        validation_command: None,
        exec_paths: vec![node],
        writable_paths: Vec::new(),
        shared_directories: &[],
        tcp_port: None,
        apparmor_network: "network unix stream,",
    }))
}
