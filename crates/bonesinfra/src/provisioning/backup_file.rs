use rustinfra::api::{Command, CommandBit, ContentUploadCommand, StringCommand};
use rustinfra::connectors::Connector;
use rustinfra::facts::files::{File, PathInfo, PathInput, Sha256File};
use rustinfra::facts::{FactExecutor, FactOptions};
use rustinfra::operations::{StatefulOperation, StatefulOperationFuture};
use sha2::{Digest, Sha256};

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

fn commands(input: &BackupPassphraseInput, remote_hash: Option<&str>, remote_file: &PathInfo) -> Vec<Command> {
    let mut commands = Vec::new();
    if remote_hash != Some(sha256_hex(input.content.as_bytes()).as_str()) {
        commands.push(Command::UploadContent(ContentUploadCommand {
            content: input.content.as_bytes().to_vec(),
            destination: input.destination.clone(),
            remote_temporary_filename: None,
        }));
    }

    let metadata_matches = matches!(
        remote_file,
        PathInfo::Present(info) if info.user == "root" && info.group == "root" && info.mode == 0o600
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

#[cfg(test)]
mod tests {
    use super::{BackupPassphraseInput, commands, sha256_hex};
    use rustinfra::api::Command;
    use rustinfra::facts::files::{FileInfo, PathInfo};

    #[test]
    fn matching_secret_and_metadata_are_a_noop() {
        let input = BackupPassphraseInput { content: "secret".to_owned(), destination: "/run/secret".to_owned() };
        let info = PathInfo::Present(FileInfo {
            user: "root".to_owned(),
            group: "root".to_owned(),
            mode: 0o600,
            accessed_at: None,
            modified_at: None,
            changed_at: None,
            size: 6,
            link_target: None,
        });

        assert!(commands(&input, Some(&sha256_hex(b"secret")), &info).is_empty());
    }

    #[test]
    fn changed_secret_is_uploaded_without_entering_a_shell_command() {
        let input = BackupPassphraseInput { content: "new secret".to_owned(), destination: "/run/secret".to_owned() };
        let commands = commands(&input, Some(&sha256_hex(b"old secret")), &PathInfo::Missing);

        assert!(matches!(&commands[0], Command::UploadContent(upload) if upload.content == b"new secret"));
        assert!(
            commands
                .iter()
                .filter_map(|command| match command {
                    Command::Shell(command) => Some(command.raw_value()),
                    _ => None,
                })
                .all(|command| !command.contains("new secret"))
        );
    }
}
