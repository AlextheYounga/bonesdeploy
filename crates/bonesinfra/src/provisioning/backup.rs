use std::path::Path;

use bonesdeploy_core::names;
use bonesdeploy_core::paths;
use rustinfra::operations::files::{DirectoryInput, TemplateInput};
use rustinfra::operations::{DeployError, DeployHost};
use serde_json::{Map, json};

use rustinfra::api::{Command, CommandBit, ContentUploadCommand, StringCommand};
use rustinfra::connectors::Connector;
use rustinfra::facts::files::{File, PathInfo, PathInput, Sha256File};
use rustinfra::facts::{FactExecutor, FactOptions};
use rustinfra::operations::{StatefulOperation, StatefulOperationFuture};
use sha2::{Digest, Sha256};

use super::operations::{directory, escaped, named, shell as add_shell, template};
use crate::ProvisioningContext;

/// Installs the in-memory Borg passphrase without placing it in argv or a
/// workstation file.
#[derive(Clone, Debug)]
pub struct BackupPassphraseInput {
    pub content: String,
    pub destination: String,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct BackupPassphraseFile;

impl StatefulOperation for BackupPassphraseFile {
    type Input = BackupPassphraseInput;

    fn name(&self) -> &'static str {
        "bonesinfra.backup_passphrase_file"
    }

    fn is_idempotent(&self) -> bool {
        true
    }

    fn commands<'a>(
        &'a self,
        input: &'a Self::Input,
        connector: &'a dyn Connector,
        options: &'a FactOptions,
    ) -> StatefulOperationFuture<'a> {
        Box::pin(async move {
            let facts = FactExecutor::new(connector, options);
            let path = PathInput::from(input.destination.clone());
            let remote_hash = facts.get(&Sha256File, &path).await?;
            let remote_file = facts.get(&File, &path).await?;
            Ok(commands(input, remote_hash.as_deref(), &remote_file))
        })
    }
}

pub(super) fn add(
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
        directory(host, name, input)?;
    }
    host.add_stateful(
        BackupPassphraseFile,
        BackupPassphraseInput {
            content: context.backup.passphrase.clone(),
            destination: context.paths.backup_passphrase_file.clone(),
        },
        named("Install root-only Borg passphrase file"),
    )?;
    let repository = escaped(&context.paths.backup_repository)?;
    let passphrase_file = escaped(&context.paths.backup_passphrase_file)?;
    add_shell(
        host,
        "Create encrypted Borg repository",
        &format!(
            "if [ ! -d {repository} ]; then BORG_PASSPHRASE=\"$(cat {passphrase_file})\" borg init --encryption repokey-blake2 {repository}; fi"
        ),
    )?;

    let mut data = Map::new();
    data.insert(names::PROJECT_NAME.to_owned(), json!(context.project_name));
    data.insert("backup_schedule".to_owned(), json!(context.backup.schedule));
    data.insert("backup_keep_days".to_owned(), json!(context.backup.retention_days));
    data.insert(names::BONESREMOTE_PATH.to_owned(), json!(context.paths.bonesremote_global_link));
    let mut cron = TemplateInput::new(cron_template, &context.paths.backup_cron_file);
    cron.data = data;
    cron.user = Some("root".to_owned());
    cron.group = Some("root".to_owned());
    cron.mode = Some(0o644);
    template(host, "Install scheduled backup cron entry", cron)?;
    Ok(())
}

fn commands(input: &BackupPassphraseInput, remote_hash: Option<&str>, remote_file: &PathInfo) -> Vec<Command> {
    let mut commands = Vec::new();
    if remote_hash != Some(sha256_hex(input.content.as_bytes()).as_str()) {
        commands.push(Command::UploadContent(ContentUploadCommand {
            content: input.content.as_bytes().to_vec(),
            destination: input.destination.clone(),
            remote_temporary_filename: None,
        }));
    }

    // RustInfra 0.2 reports parsed permission digits as decimal (600), while
    // an octal value is also accepted if that representation is corrected.
    let metadata_matches = matches!(
        remote_file,
        PathInfo::Present(info) if info.user == "root" && info.group == "root" && matches!(info.mode, 600 | 0o600)
    );
    if !metadata_matches {
        commands.push(shell(["chown".into(), "root:root".into(), CommandBit::quoted(input.destination.as_str())]));
        commands.push(shell(["chmod".into(), "600".into(), CommandBit::quoted(input.destination.as_str())]));
    }
    commands
}

fn sha256_hex(value: &[u8]) -> String {
    let digest = Sha256::digest(value);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn shell(bits: impl IntoIterator<Item = CommandBit>) -> Command {
    Command::Shell(StringCommand::new(bits))
}
