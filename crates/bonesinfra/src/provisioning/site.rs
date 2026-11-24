use bonesdeploy_core::names;

use std::path::{Path, PathBuf};

use anyhow::Result;
use bonesdeploy_core::config::Bones;
use rustinfra::operations::files::{DirectoryInput, TemplateInput};
use rustinfra::operations::server::{Group, GroupInput, User, UserInput};
use rustinfra::operations::{DeployError, DeployHost, PreparedDeploy};
use serde_json::{Map, Value, json};

use super::backup;
use super::operations::{directory, escaped, named, shell, template as register_template};
use crate::{
    ProvisioningContext, ProvisioningReport, ProvisioningRuntime, TemplateResolver, reject_custom_provisioning,
};

struct SiteTemplates {
    placeholder: PathBuf,
    backup_cron: PathBuf,
    etckeeper_record: PathBuf,
}

/// Prepares the complete base-site plan without opening an SSH connection.
///
/// # Errors
/// Returns an error when a required project-local template is missing or the
/// RustInfra operation graph cannot be constructed.
pub fn prepare_site_setup(
    context: &ProvisioningContext,
    runtime: &ProvisioningRuntime,
    templates: &TemplateResolver,
) -> Result<PreparedDeploy> {
    let templates = SiteTemplates {
        placeholder: templates.resolve(names::INDEX_TEMPLATE)?,
        backup_cron: templates.resolve("shared/cron/backup.cron.j2")?,
        etckeeper_record: templates.resolve(names::ETCKEEPER_TEMPLATE)?,
    };
    let template_data = context.template_data()?;
    runtime.prepare(|host| {
        add_identities(host, context)?;
        add_directories(host, context)?;
        add_placeholder(host, context, &templates.placeholder, &template_data)?;
        if context.backup.is_configured() {
            backup::add(host, context, &templates.backup_cron)?;
        }
        add_etckeeper_record(host, &templates.etckeeper_record, "BonesInfra site setup")?;
        Ok(())
    })
}

/// Provisions the base site directly through the Rust BonesInfra library.
///
/// # Errors
/// Returns an error for unsupported custom provisioning, invalid configuration,
/// missing templates, SSH failures, or failed operations.
pub async fn apply_site_setup(config: &Bones, project_root: &Path) -> Result<ProvisioningReport> {
    reject_custom_provisioning(project_root)?;
    let context = ProvisioningContext::from_bones(config)?;
    let runtime = ProvisioningRuntime::new(&context.server)?;
    let deploy = prepare_site_setup(&context, &runtime, &TemplateResolver::new(project_root))?;
    runtime.execute(&deploy).await
}

fn add_identities(host: &mut DeployHost<'_>, context: &ProvisioningContext) -> Result<(), DeployError> {
    let mut group = GroupInput::new(&context.runtime_group);
    group.system = true;
    host.add_stateful(Group, group, named("Ensure runtime group exists"))?;

    let mut user = UserInput::new(&context.runtime_user);
    user.system = true;
    user.home = Some("/nonexistent".to_owned());
    user.shell = Some(names::NOLOGIN_SHELL.to_owned());
    user.group = Some(context.runtime_group.clone());
    user.groups = vec![context.runtime_group.clone()];
    user.append = true;
    user.ensure_home = false;
    user.create_home = false;
    host.add_stateful(User, user, named("Ensure runtime user exists with groups"))?;
    Ok(())
}

fn add_directories(host: &mut DeployHost<'_>, context: &ProvisioningContext) -> Result<(), DeployError> {
    let definitions = [
        (
            "Ensure control-plane site state directory exists",
            &context.paths.site_root,
            Some("root"),
            Some("root"),
            0o700,
        ),
        ("Ensure project root parent directory is traversable", &context.paths.project_root_parent, None, None, 0o711),
        ("Ensure project root boundary exists", &context.paths.project_root, Some("root"), Some("root"), 0o751),
        (
            "Ensure releases directory with setgid",
            &context.paths.releases,
            Some("root"),
            Some(context.runtime_group.as_str()),
            0o2750,
        ),
        (
            "Ensure shared directory (owned by runtime user)",
            &context.paths.shared,
            Some(context.runtime_user.as_str()),
            Some(context.runtime_group.as_str()),
            0o750,
        ),
        (
            "Ensure placeholder release directory exists",
            &context.paths.placeholder_web_root,
            Some("root"),
            Some(context.runtime_group.as_str()),
            0o750,
        ),
    ];
    for (name, path, user, group, mode) in definitions {
        let mut input = DirectoryInput::new(path);
        input.user = user.map(str::to_owned);
        input.group = group.map(str::to_owned);
        input.mode = Some(mode);
        directory(host, name, input)?;
    }
    Ok(())
}

fn add_placeholder(
    host: &mut DeployHost<'_>,
    context: &ProvisioningContext,
    template: &Path,
    template_data: &Map<String, Value>,
) -> Result<(), DeployError> {
    let mut input = TemplateInput::new(template, &context.paths.placeholder_index);
    input.data = template_data.clone();
    input.user = Some("root".to_owned());
    input.group = Some(context.runtime_group.clone());
    input.mode = Some(0o640);
    register_template(host, "Seed placeholder index page", input)?;

    let current = escaped(&context.paths.current)?;
    let placeholder = escaped(&context.paths.placeholder_release)?;
    shell(
        host,
        "Point current symlink at placeholder release when none exists",
        &format!("test -e {current} -o -L {current} || ln -s {placeholder} {current}"),
    )?;
    Ok(())
}

fn add_etckeeper_record(host: &mut DeployHost<'_>, template: &Path, message: &str) -> Result<(), DeployError> {
    use rustinfra::operations::server::{ScriptTemplate, ScriptTemplateInput};

    let mut data = Map::new();
    data.insert("message".to_owned(), json!(message));
    host.add_stateful(
        ScriptTemplate,
        ScriptTemplateInput { source: template.to_path_buf(), arguments: Vec::new(), data },
        named(names::ETCKEEPER_RECORD_OPERATION),
    )?;
    Ok(())
}
