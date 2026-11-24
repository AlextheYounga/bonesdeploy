use bonesdeploy_core::names;

use anyhow::Result;

use super::{Language, Placeholder, RuntimePlan, ServerPlan, bool_value, numeric_version, port_value};
use crate::ProvisioningContext;

pub(super) fn plan(context: &ProvisioningContext) -> Result<RuntimePlan> {
    numeric_version(&context.node_version, 3, names::NODE_VERSION)?;
    if bool_value(context, names::IS_STATIC, true)? {
        return Ok(RuntimePlan::Static {
            root: "out",
            nginx_template: "frameworks/next/nginx/static-site-nginx.conf.j2",
            placeholder_template: "frameworks/next/nginx/index.html.j2",
        });
    }
    let port = port_value(context, "internal_port", 3100)?;
    let node = names::node_binary(&context.node_version);
    Ok(RuntimePlan::Server(ServerPlan {
        name: "next",
        label: "Next.js app server",
        language: Language::Node,
        nginx_template: "frameworks/next/nginx/app-site-nginx.conf.j2",
        apparmor_template: "frameworks/next/app-profile.j2",
        placeholder: Placeholder::Next,
        command: format!(
            "/usr/bin/env NODE_ENV=production PORT={port} HOSTNAME=127.0.0.1 {node} .next/standalone/server.js"
        ),
        validation_command: Some("test -f .next/standalone/server.js".to_owned()),
        exec_paths: vec![node],
        writable_paths: Vec::new(),
        shared_directories: &[],
        tcp_port: Some(port),
        apparmor_network: "network inet stream,",
    }))
}
