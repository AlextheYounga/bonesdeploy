use rustinfra::operations::files::{Directory, DirectoryInput, Link, LinkInput, Template, TemplateInput};
use rustinfra::operations::server::{Shell, ShellCommands};
use rustinfra::operations::systemd::{Service, ServiceInput};
use rustinfra::operations::{DeployError, DeployHost, OperationId, OperationOptions};

use crate::shell_quote;

pub(super) fn named(name: &str) -> OperationOptions {
    OperationOptions { name: Some(name.to_owned()), ..OperationOptions::default() }
}

pub(super) fn shell(host: &mut DeployHost<'_>, name: &str, command: &str) -> Result<(), DeployError> {
    host.add_with_options(Shell, ShellCommands::from(command), named(name))?;
    Ok(())
}

pub(super) fn template(host: &mut DeployHost<'_>, name: &str, input: TemplateInput) -> Result<(), DeployError> {
    host.add_stateful(Template, input, named(name))?;
    Ok(())
}

pub(super) fn directory(host: &mut DeployHost<'_>, name: &str, input: DirectoryInput) -> Result<(), DeployError> {
    host.add_stateful(Directory, input, named(name))?;
    Ok(())
}

pub(super) fn link(host: &mut DeployHost<'_>, name: &str, input: LinkInput) -> Result<(), DeployError> {
    host.add_stateful(Link, input, named(name))?;
    Ok(())
}

pub(super) fn service(host: &mut DeployHost<'_>, name: &str, input: ServiceInput) -> Result<(), DeployError> {
    host.add_stateful(Service, input, named(name))?;
    Ok(())
}

pub(super) fn escaped(value: &str) -> Result<String, DeployError> {
    shell_quote::quote(value)
        .map_err(|_| DeployError::InconsistentOperation { id: OperationId::new("invalid-shell-argument") })
}
