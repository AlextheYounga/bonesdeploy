use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use bonesdeploy_core::{config::Bones, names, paths};
use bonesinfra::provisioning::{prepare_runtime, prepare_site_setup};
use bonesinfra::{ProvisioningContext, ProvisioningRuntime, TemplateResolver};
use rustinfra::api::{Config, RsyncCommand, State, StringCommand};
use rustinfra::connectors::{
    CommandOutput, CommandResult, Connector, ConnectorError, ConnectorFuture, ConnectorTransferFuture,
    FileTransferOptions, LocalConnector, RunOptions,
};
use rustinfra::operations::{DeployRuntime, ExecutionStrategy};
use sha2::{Digest, Sha256};

#[derive(Default)]
struct RecordingConnector {
    commands: Mutex<Vec<String>>,
    uploads: Mutex<Vec<(String, Vec<u8>)>>,
    links: Vec<String>,
    existing_secret: Option<(String, String)>,
}

impl Connector for RecordingConnector {
    fn connect(&self) -> ConnectorTransferFuture<'_> {
        Box::pin(async { Ok(()) })
    }
    fn disconnect(&self) -> ConnectorTransferFuture<'_> {
        Box::pin(async { Ok(()) })
    }
    fn run_shell_command<'a>(&'a self, command: StringCommand, _options: &'a RunOptions) -> ConnectorFuture<'a> {
        let raw = command.raw_value();
        self.commands.lock().unwrap_or_else(|e| e.into_inner()).push(raw.clone());
        let is_link = raw.starts_with("! (test -e ") && self.links.iter().any(|path| raw.contains(path));
        let response = self.existing_secret.as_ref().and_then(|(path, hash)| {
            if !raw.contains(path) {
                return None;
            }
            if raw.contains("sha256sum < ") {
                Some(hash.clone())
            } else if raw.starts_with("! (test -e ") {
                Some("user=root group=root mode=-rw------- atime=0 mtime=0 ctime=0 size=6 secret".to_owned())
            } else {
                None
            }
        });
        Box::pin(async move {
            if is_link || response.is_some() {
                let line = response.unwrap_or_else(|| {
                    "user=root group=root mode=lrwxrwxrwx atime=0 mtime=0 ctime=0 size=1 link -> target".to_owned()
                });
                let result = LocalConnector
                    .run_shell_command(StringCommand::new([format!("printf '%s\\n' '{line}'")]), &RunOptions::default())
                    .await?;
                return Ok(result);
            }
            Ok(CommandResult { success: true, exit_code: Some(0), output: CommandOutput::default() })
        })
    }
    fn put_file<'a>(
        &'a self,
        source: &'a Path,
        destination: &'a str,
        _options: FileTransferOptions<'a>,
    ) -> ConnectorTransferFuture<'a> {
        Box::pin(async move {
            let content =
                fs::read(source).map_err(|error| ConnectorError::Connection { message: error.to_string() })?;
            self.uploads.lock().unwrap_or_else(|e| e.into_inner()).push((destination.to_owned(), content));
            Ok(())
        })
    }
    fn get_file<'a>(
        &'a self,
        _source: &'a str,
        _destination: &'a Path,
        _options: FileTransferOptions<'a>,
    ) -> ConnectorTransferFuture<'a> {
        Box::pin(async { Ok(()) })
    }
    fn rsync<'a>(&'a self, _command: &'a RsyncCommand, _options: &'a RunOptions) -> ConnectorTransferFuture<'a> {
        Box::pin(async { Ok(()) })
    }
}

#[tokio::test]
async fn runtime_router_enables_tls_only_with_both_certificate_links() -> Result<()> {
    for (certificate_present, key_present) in [(false, false), (true, false), (false, true), (true, true)] {
        let project = tempfile::tempdir()?;
        let mut config = Bones::for_site("atlas");
        config.host = "server.example.com".into();
        config.domain = "app.example.com".into();
        config.runtime.template = "vue".into();
        bonesinfra::materialize_project_artifacts(project.path(), &config.runtime)?;
        fs::write(
            project.path().join("infra/templates").join(names::ROUTER_TEMPLATE),
            "{% if nginx_ssl_enabled %}tls{% else %}http{% endif %}\n",
        )?;
        let context = ProvisioningContext::from_bones(&config)?;
        let provisioning = ProvisioningRuntime::new(&context.server)?;
        let deploy = prepare_runtime(&context, &provisioning, &TemplateResolver::new(project.path()))?;
        let mut connector = RecordingConnector::default();
        if certificate_present {
            connector.links.push(paths::ssl_certificate_path(&context.domain));
        }
        if key_present {
            connector.links.push(paths::ssl_certificate_key_path(&context.domain));
        }
        let connector = Arc::new(connector);
        let selected: Arc<dyn Connector> = connector.clone();
        let runtime = DeployRuntime::new([(context.server.host.clone(), selected)].into_iter().collect());
        let mut state = State::new(provisioning.inventory().clone(), Config::default());
        runtime.execute(&mut state, &deploy, ExecutionStrategy::Lockstep).await?;
        let uploads = connector.uploads.lock().unwrap_or_else(|e| e.into_inner());
        let (_, contents) = uploads
            .iter()
            .find(|(path, _)| path == &context.paths.nginx_site_available)
            .ok_or_else(|| anyhow::anyhow!("router was not uploaded"))?;
        assert_eq!(contents.as_slice(), if certificate_present && key_present { b"tls".as_slice() } else { b"http" });
    }
    Ok(())
}

#[tokio::test]
async fn site_setup_uploads_the_backup_passphrase_without_putting_it_in_shell_commands() -> Result<()> {
    let project = tempfile::tempdir()?;
    for (path, content) in [
        (names::INDEX_TEMPLATE, "{{ project_name }}"),
        ("shared/cron/backup.cron.j2", "{{ backup_schedule }} root {{ bonesremote_path }}"),
        (names::ETCKEEPER_TEMPLATE, "#!/bin/sh\n"),
    ] {
        let path = project.path().join("infra/templates").join(path);
        fs::create_dir_all(path.parent().ok_or_else(|| anyhow::anyhow!("missing template parent"))?)?;
        fs::write(path, content)?;
    }
    let mut config = Bones::for_site("example");
    config.host = "server.example.com".into();
    config.ssh_user = "deploy".into();
    config.runtime.template = "laravel".into();
    config.backup.passphrase = "a secret that must not be shell text".into();
    let context = ProvisioningContext::from_bones(&config)?;
    let provisioning = ProvisioningRuntime::new(&context.server)?;
    let deploy = prepare_site_setup(&context, &provisioning, &TemplateResolver::new(project.path()))?;
    let connector = Arc::new(RecordingConnector::default());
    let selected: Arc<dyn Connector> = connector.clone();
    let runtime = DeployRuntime::new([(context.server.host.clone(), selected)].into_iter().collect());
    let mut state = State::new(provisioning.inventory().clone(), Config::default());
    runtime.execute(&mut state, &deploy, ExecutionStrategy::Lockstep).await?;
    {
        let uploads = connector.uploads.lock().unwrap_or_else(|e| e.into_inner());
        assert!(uploads.iter().any(|(path, content)| path == &context.paths.backup_passphrase_file
            && content == config.backup.passphrase.as_bytes()));
        let commands = connector.commands.lock().unwrap_or_else(|e| e.into_inner());
        assert!(commands.iter().all(|command| !command.contains(&config.backup.passphrase)));
    }

    let connector = Arc::new(RecordingConnector {
        existing_secret: Some((
            context.paths.backup_passphrase_file.clone(),
            format!("{:x}", Sha256::digest(config.backup.passphrase.as_bytes())),
        )),
        ..RecordingConnector::default()
    });
    let selected: Arc<dyn Connector> = connector.clone();
    let runtime = DeployRuntime::new([(context.server.host.clone(), selected)].into_iter().collect());
    let mut state = State::new(provisioning.inventory().clone(), Config::default());
    runtime.execute(&mut state, &deploy, ExecutionStrategy::Lockstep).await?;
    let uploads = connector.uploads.lock().unwrap_or_else(|e| e.into_inner());
    assert!(
        !uploads.iter().any(|(path, _)| path == &context.paths.backup_passphrase_file),
        "{:?}",
        connector
            .commands
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .filter(|cmd| cmd.contains(&context.paths.backup_passphrase_file))
            .collect::<Vec<_>>()
    );
    let commands = connector.commands.lock().unwrap_or_else(|e| e.into_inner());
    assert!(!commands.iter().any(|command| (command.starts_with("chown ") || command.starts_with("chmod "))
        && command.contains(&context.paths.backup_passphrase_file)));
    Ok(())
}
