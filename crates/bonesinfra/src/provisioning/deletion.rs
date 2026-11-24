use std::borrow::Cow;
use std::cmp::Reverse;
use std::path::Path;

use anyhow::Result;
use bonesdeploy_core::config::Bones;
use rustinfra::operations::files::{Directory, DirectoryInput};
use rustinfra::operations::server::{Shell, ShellCommands};
use rustinfra::operations::systemd::{DaemonReload, DaemonReloadInput, Service, ServiceInput};
use rustinfra::operations::{DeployError, DeployHost, OperationOptions, PreparedDeploy};

use crate::manifest::{ArtifactKind, DeletionPlan};
use crate::{ProvisioningContext, ProvisioningReport, ProvisioningRuntime, reject_custom_provisioning};

/// Prepares the validated site removal sequence without connecting.
///
/// # Errors
/// Returns an error when the operation graph cannot be constructed.
pub fn prepare_site_delete(runtime: &ProvisioningRuntime, plan: &DeletionPlan) -> Result<PreparedDeploy> {
    runtime.prepare(|host| {
        stop_services(host, plan)?;
        remove_artifacts(host, plan)?;
        if plan.artifacts.iter().any(|artifact| artifact.owner == "tunnel") {
            host.add_with_options(
                DaemonReload,
                DaemonReloadInput::default(),
                named("Reload systemd after Quick Tunnel removal"),
            )?;
            let mut nginx = ServiceInput::new("nginx");
            nginx.running = None;
            nginx.reloaded = true;
            host.add_stateful(Service, nginx, named("Reload nginx after Quick Tunnel removal"))?;
        }
        Ok(())
    })
}

/// Removes a site directly through RustInfra after revalidating its persisted
/// deletion plan against the current project configuration.
///
/// # Errors
/// Returns an error for custom provisioning, an unsafe or malformed plan, or a
/// failed RustInfra operation.
pub async fn delete_site(config: &Bones, project_root: &Path, persisted_plan: &str) -> Result<ProvisioningReport> {
    reject_custom_provisioning(project_root)?;
    let context = ProvisioningContext::from_bones(config)?;
    let plan = DeletionPlan::parse(persisted_plan, &context)?;
    let runtime = ProvisioningRuntime::new(&context.server)?;
    let deploy = prepare_site_delete(&runtime, &plan)?;
    runtime.execute(&deploy).await
}

fn stop_services(host: &mut DeployHost<'_>, plan: &DeletionPlan) -> Result<(), DeployError> {
    for service in &plan.services {
        if service.owner == "tunnel" {
            let unit = escaped(&service.unit);
            shell(
                host,
                &format!("Stop and disable {} when installed", service.name),
                &format!("if systemctl cat -- {unit} >/dev/null 2>&1; then systemctl disable --now -- {unit}; fi"),
            )?;
            continue;
        }
        let mut input = ServiceInput::new(&service.unit);
        input.running = Some(false);
        input.enabled = Some(false);
        host.add_stateful(Service, input, named(&format!("Stop and disable {}", service.name)))?;
    }
    Ok(())
}

fn remove_artifacts(host: &mut DeployHost<'_>, plan: &DeletionPlan) -> Result<(), DeployError> {
    let mut artifacts = plan.artifacts.iter().collect::<Vec<_>>();
    artifacts.sort_by_key(|artifact| Reverse(artifact.path.len()));
    for artifact in artifacts {
        let name = format!("Remove {}", artifact.name);
        if artifact.kind == ArtifactKind::Directory {
            let mut input = DirectoryInput::new(&artifact.path);
            input.present = false;
            input.recursive = true;
            host.add_stateful(Directory, input, named(&name))?;
        } else {
            shell(host, &name, &format!("rm -f -- {}", escaped(&artifact.path)))?;
        }
    }
    Ok(())
}

fn shell(host: &mut DeployHost<'_>, name: &str, command: &str) -> Result<(), DeployError> {
    host.add_with_options(Shell, ShellCommands::from(command), named(name))?;
    Ok(())
}

fn escaped(value: &str) -> String {
    shell_escape::escape(Cow::Borrowed(value)).into_owned()
}

fn named(name: &str) -> OperationOptions {
    OperationOptions { name: Some(name.to_owned()), ..OperationOptions::default() }
}
