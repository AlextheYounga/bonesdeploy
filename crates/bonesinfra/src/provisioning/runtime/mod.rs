use std::path::Path;

use anyhow::Result;
use bonesdeploy_core::config::{Bones, RuntimeBackend};
use rustinfra::operations::PreparedDeploy;

use crate::frameworks;
use crate::{
    ProvisioningContext, ProvisioningReport, ProvisioningRuntime, TemplateResolver, reject_custom_provisioning,
};

mod common;
mod compose;
mod native;

/// Prepares the selected native framework or Compose runtime without
/// connecting to the target host.
///
/// # Errors
/// Returns an error for invalid framework values, missing project templates,
/// or an invalid operation graph.
pub fn prepare_runtime(
    context: &ProvisioningContext,
    runtime: &ProvisioningRuntime,
    templates: &TemplateResolver,
) -> Result<PreparedDeploy> {
    if context.backend == RuntimeBackend::Docker {
        let assets = compose::Assets::resolve(templates)?;
        let template_data = context.template_data()?;
        return runtime.prepare(|host| compose::add(host, context, &template_data, &assets));
    }

    let plan = frameworks::plan(context)?;
    if plan == frameworks::RuntimePlan::Custom {
        return runtime.prepare(|_| Ok(()));
    }
    let common_assets = common::Assets::resolve(templates)?;
    let native_assets = native::Assets::resolve(templates, &plan)?;
    let template_data = context.template_data()?;
    let uses_tcp = matches!(&plan, frameworks::RuntimePlan::Server(server) if server.tcp_port.is_some());
    runtime.prepare(|host| {
        common::setup(host, context, &template_data, &common_assets, uses_tcp)?;
        native::add(host, context, &template_data, &plan, &native_assets)?;
        common::finish(host, context, &common_assets)?;
        Ok(())
    })
}

/// Applies the selected runtime directly through RustInfra.
///
/// # Errors
/// Returns an error for existing custom provisioning, invalid configuration,
/// missing templates, or failed RustInfra operations.
pub async fn provision_runtime(config: &Bones, project_root: &Path) -> Result<ProvisioningReport> {
    reject_custom_provisioning(project_root)?;
    let context = ProvisioningContext::from_bones(config)?;
    let runtime = ProvisioningRuntime::new(&context.server)?;
    let deploy = prepare_runtime(&context, &runtime, &TemplateResolver::new(project_root))?;
    runtime.execute(&deploy).await
}
