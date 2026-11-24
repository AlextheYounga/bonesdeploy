use bonesdeploy_core::state;

use std::path::{Path, PathBuf};

use anyhow::Result;
use bonesdeploy_core::config::Bones;
use bonesdeploy_core::paths;
use rustinfra::operations::files::{Directory, DirectoryInput, Template, TemplateInput};
use rustinfra::operations::server::{Group, GroupInput, Shell, ShellCommands, User, UserInput};
use rustinfra::operations::{DeployError, DeployHost, OperationOptions, PreparedDeploy};
use serde_json::{Map, Value, json};

use super::backup_file::{BackupPassphraseFile, BackupPassphraseInput};
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
        placeholder: templates.resolve(state::INDEX_TEMPLATE)?,
        backup_cron: templates.resolve("shared/cron/backup.cron.j2")?,
        etckeeper_record: templates.resolve(state::ETCKEEPER_TEMPLATE)?,
    };
    let template_data = context.template_data()?;
    runtime.prepare(|host| {
        add_identities(host, context)?;
        add_directories(host, context)?;
        add_placeholder(host, context, &templates.placeholder, &template_data)?;
        if context.backup.is_configured() {
            add_backup(host, context, &templates.backup_cron)?;
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
pub async fn site_setup(config: &Bones, project_root: &Path) -> Result<ProvisioningReport> {
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
    user.shell = Some(state::NOLOGIN_SHELL.to_owned());
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
        host.add_stateful(Directory, input, named(name))?;
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
    host.add_stateful(Template, input, named("Seed placeholder index page"))?;

    host.add_with_options(
        Shell,
        ShellCommands::from(format!(
            "test -e '{}' -o -L '{}' || ln -s '{}' '{}'",
            context.paths.current, context.paths.current, context.paths.placeholder_release, context.paths.current
        )),
        named("Point current symlink at placeholder release when none exists"),
    )?;
    Ok(())
}

fn add_backup(
    host: &mut DeployHost<'_>,
    context: &ProvisioningContext,
    cron_template: &Path,
) -> Result<(), DeployError> {
    for (name, path, mode) in [
        ("Ensure backup repository root exists", paths::BACKUPS_ROOT, 0o700),
        ("Ensure BonesRemote site state directory exists", context.paths.site_root.as_str(), 0o700),
    ] {
        let mut input = DirectoryInput::new(path);
        input.mode = Some(mode);
        host.add_stateful(Directory, input, named(name))?;
    }
    host.add_stateful(
        BackupPassphraseFile,
        BackupPassphraseInput {
            content: context.backup.passphrase.clone(),
            destination: context.paths.backup_passphrase_file.clone(),
        },
        named("Install root-only Borg passphrase file"),
    )?;
    host.add_with_options(
        Shell,
        ShellCommands::from(format!(
            "if [ ! -d '{}' ]; then BORG_PASSPHRASE=\"$(cat '{}')\" borg init --encryption repokey-blake2 '{}'; fi",
            context.paths.backup_repository, context.paths.backup_passphrase_file, context.paths.backup_repository
        )),
        named("Create encrypted Borg repository"),
    )?;

    let mut data = Map::new();
    data.insert(state::PROJECT_NAME.to_owned(), json!(context.project_name));
    data.insert("backup_schedule".to_owned(), json!(context.backup.schedule));
    data.insert("backup_keep_days".to_owned(), json!(context.backup.retention_days));
    data.insert(state::BONESREMOTE_PATH.to_owned(), json!(context.paths.bonesremote_global_link));
    let mut cron = TemplateInput::new(cron_template, &context.paths.backup_cron_file);
    cron.data = data;
    cron.user = Some("root".to_owned());
    cron.group = Some("root".to_owned());
    cron.mode = Some(0o644);
    host.add_stateful(Template, cron, named("Install scheduled backup cron entry"))?;
    Ok(())
}

fn add_etckeeper_record(host: &mut DeployHost<'_>, template: &Path, message: &str) -> Result<(), DeployError> {
    use rustinfra::operations::server::{ScriptTemplate, ScriptTemplateInput};

    let mut data = Map::new();
    data.insert("message".to_owned(), json!(message));
    host.add_stateful(
        ScriptTemplate,
        ScriptTemplateInput { source: template.to_path_buf(), arguments: Vec::new(), data },
        named(state::ETCKEEPER_RECORD_OPERATION),
    )?;
    Ok(())
}

fn named(name: &str) -> OperationOptions {
    OperationOptions { name: Some(name.to_owned()), ..OperationOptions::default() }
}
