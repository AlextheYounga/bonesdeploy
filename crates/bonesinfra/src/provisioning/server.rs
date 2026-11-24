use bonesdeploy_core::state;

use std::error::Error;
use std::path::{Path, PathBuf};

use anyhow::Result;
use bonesdeploy_core::config::Bones;
use bonesdeploy_core::paths;
use rustinfra::connectors::Connector;
use rustinfra::facts::server::{Arch, LinuxDistribution, LinuxDistributionInfo};
use rustinfra::facts::{FactExecutor, FactOptions};
use rustinfra::operations::apt::{Packages, PackagesInput};
use rustinfra::operations::files::{Directory, DirectoryInput, Template, TemplateInput};
use rustinfra::operations::server::{ScriptTemplate, ScriptTemplateInput, Shell, ShellCommands, User, UserInput};
use rustinfra::operations::systemd::{Service, ServiceInput};
use rustinfra::operations::{DeployError, DeployHost, OperationOptions, PreparedDeploy};
use rustinfra::operations::{StatefulOperation, StatefulOperationFuture};
use serde_json::{Map, Value, json};

use crate::{ProvisioningReport, ProvisioningRuntime, ServerContext, TemplateResolver, reject_custom_provisioning};

const BASE_PACKAGES: &[&str] = &[
    "build-essential",
    "borgbackup",
    "ca-certificates",
    "fail2ban",
    "curl",
    "git",
    "gpg",
    "rsync",
    "sudo",
    "nginx",
    "openssl",
    "apparmor",
    "apparmor-utils",
    "unattended-upgrades",
    "certbot",
    "ufw",
    "etckeeper",
    "acl",
    "htop",
    "nano",
    "sqlite3",
    "unzip",
    "zip",
    "zsh",
];

struct ServerTemplates {
    disable_algif: PathBuf,
    fail2ban: PathBuf,
    auto_upgrades: PathBuf,
    unattended_upgrades: PathBuf,
    authorized_keys: PathBuf,
    install_bonesremote: PathBuf,
    sshd: PathBuf,
    sudoers: PathBuf,
    etckeeper_record: PathBuf,
}

/// Prepares the complete server baseline without connecting to the host.
///
/// # Errors
/// Returns an error for missing templates or an invalid operation graph.
pub fn prepare_server_setup(
    server: &ServerContext,
    runtime: &ProvisioningRuntime,
    templates: &TemplateResolver,
    bonesremote_version: &str,
) -> Result<PreparedDeploy> {
    let templates = ServerTemplates {
        disable_algif: templates.resolve("shared/modprobe/disable-algif.conf.j2")?,
        fail2ban: templates.resolve("shared/fail2ban/jail.local.j2")?,
        auto_upgrades: templates.resolve("shared/unattended-upgrades/20auto-upgrades.j2")?,
        unattended_upgrades: templates.resolve("shared/unattended-upgrades/50unattended-upgrades.j2")?,
        authorized_keys: templates.resolve("shared/scripts/copy-ssh-authorized-keys.sh.j2")?,
        install_bonesremote: templates.resolve("shared/scripts/install-bonesremote.sh.j2")?,
        sshd: templates.resolve("shared/sshd/99-bonesdeploy-root-login.conf.j2")?,
        sudoers: templates.resolve("shared/sudoers/bonesdeploy.j2")?,
        etckeeper_record: templates.resolve(state::ETCKEEPER_TEMPLATE)?,
    };
    runtime.prepare(|host| {
        host.add_stateful(SupportedHost, (), named("Validate supported production host"))?;
        add_packages(host)?;
        shell(host, "Initialize etckeeper in /etc", "etckeeper init")?;
        add_service(host, "Ensure apparmor service is enabled and started", "apparmor")?;
        add_template(
            host,
            "Disable the vulnerable algif_aead module",
            template_input(
                &templates.disable_algif,
                "/etc/modprobe.d/disable-algif.conf",
                Map::new(),
                None,
            ),
        )?;
        shell(
            host,
            "Unload the vulnerable algif_aead module",
            "if ! systemd-detect-virt --container --quiet && grep -q '^algif_aead ' /proc/modules; then rmmod algif_aead; fi",
        )?;
        add_firewall(host, server.port)?;
        add_fail2ban(host, server.port, &templates.fail2ban)?;
        add_unattended_upgrades(host, &templates)?;
        add_deploy_user(host, server, &templates.authorized_keys)?;
        add_sshd_hardening(host, &templates.sshd)?;
        add_bonesremote(host, &templates.install_bonesremote, bonesremote_version)?;
        add_sudoers(host, &templates.sudoers)?;
        add_etckeeper_record(host, &templates.etckeeper_record)?;
        Ok(())
    })
}

#[derive(Clone, Copy)]
struct SupportedHost;

impl StatefulOperation for SupportedHost {
    type Input = ();

    fn name(&self) -> &'static str {
        "bonesinfra.supported_host"
    }

    fn is_idempotent(&self) -> bool {
        true
    }

    fn commands<'a>(
        &'a self,
        _input: &'a Self::Input,
        connector: &'a dyn Connector,
        options: &'a FactOptions,
    ) -> StatefulOperationFuture<'a> {
        Box::pin(async move {
            let facts = FactExecutor::new(connector, options);
            let distribution = facts.get(&LinuxDistribution, &()).await?;
            let architecture = facts.get(&Arch, &()).await?;
            validate_supported_host(&distribution, architecture.as_deref())?;
            Ok(Vec::new())
        })
    }
}

fn validate_supported_host(
    distribution: &LinuxDistributionInfo,
    architecture: Option<&str>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let supported = architecture == Some(state::X86_64)
        && match distribution.release_meta.get("ID").map(String::as_str) {
            Some("debian") => distribution.major.is_some_and(|major| major >= 12),
            Some("ubuntu") => distribution.major.zip(distribution.minor).is_some_and(|version| version >= (24, 4)),
            _ => false,
        };
    if supported { Ok(()) } else { Err("production hosts must be Debian 12+ or Ubuntu 24.04+ on x86_64".into()) }
}

/// Applies the server baseline directly through RustInfra.
///
/// # Errors
/// Returns an error for unsupported custom provisioning, invalid connection
/// data, missing templates, or failed provisioning operations.
pub async fn server_setup(
    config: &Bones,
    project_root: &Path,
    bonesremote_version: &str,
) -> Result<ProvisioningReport> {
    reject_custom_provisioning(project_root)?;
    let server = ServerContext::from_bones(config)?;
    let runtime = ProvisioningRuntime::new(&server)?;
    let deploy = prepare_server_setup(&server, &runtime, &TemplateResolver::new(project_root), bonesremote_version)?;
    runtime.execute(&deploy).await
}

fn add_packages(host: &mut DeployHost<'_>) -> Result<(), DeployError> {
    let mut input = PackagesInput::new(BASE_PACKAGES.iter().copied());
    input.update = true;
    host.add_stateful(Packages, input, named("Install setup apt packages"))?;
    Ok(())
}

fn add_service(host: &mut DeployHost<'_>, name: &str, service: &str) -> Result<(), DeployError> {
    let mut input = ServiceInput::new(service);
    input.enabled = Some(true);
    host.add_stateful(Service, input, named(name))?;
    Ok(())
}

fn add_firewall(host: &mut DeployHost<'_>, ssh_port: u16) -> Result<(), DeployError> {
    for command in [
        format!("ufw allow {ssh_port}/tcp"),
        "ufw allow 80/tcp".to_owned(),
        "ufw allow 443/tcp".to_owned(),
        "ufw --force default deny incoming".to_owned(),
        "ufw --force default allow outgoing".to_owned(),
        "ufw --force enable".to_owned(),
    ] {
        shell(host, "Apply UFW configuration", &command)?;
    }
    shell(host, "Display UFW status", "ufw status verbose")
}

fn add_fail2ban(host: &mut DeployHost<'_>, ssh_port: u16, template: &Path) -> Result<(), DeployError> {
    let mut data = Map::new();
    data.insert(state::SSH_PORT.to_owned(), json!(ssh_port));
    add_template(
        host,
        "Install fail2ban jail.local",
        template_input(template, "/etc/fail2ban/jail.local", data, None),
    )?;
    add_service(host, "Ensure fail2ban service is enabled and started", "fail2ban")
}

fn add_unattended_upgrades(host: &mut DeployHost<'_>, templates: &ServerTemplates) -> Result<(), DeployError> {
    add_template(
        host,
        "Install apt auto-upgrades config",
        template_input(&templates.auto_upgrades, state::APT_AUTO_UPGRADES, Map::new(), None),
    )?;
    add_template(
        host,
        "Install unattended-upgrades config",
        template_input(&templates.unattended_upgrades, state::APT_UNATTENDED_UPGRADES, Map::new(), None),
    )
}

fn add_deploy_user(host: &mut DeployHost<'_>, server: &ServerContext, template: &Path) -> Result<(), DeployError> {
    let mut user = UserInput::new(paths::DEPLOY_USER);
    user.shell = Some("/bin/bash".to_owned());
    user.ensure_home = true;
    user.create_home = true;
    host.add_stateful(User, user, named("Ensure deploy user exists"))?;

    let data = Map::from_iter([
        (state::DEPLOY_USER.to_owned(), json!(paths::DEPLOY_USER)),
        (state::SSH_USER.to_owned(), json!(server.ssh_user)),
    ]);
    host.add_stateful(
        ScriptTemplate,
        ScriptTemplateInput { source: template.to_path_buf(), arguments: Vec::new(), data },
        named(&format!("Copy {} SSH key to deploy user {}", server.ssh_user, paths::DEPLOY_USER)),
    )?;
    Ok(())
}

fn add_sshd_hardening(host: &mut DeployHost<'_>, template: &Path) -> Result<(), DeployError> {
    let path = format!("{}/99-bonesdeploy-root-login.conf", paths::ETC_SSH_SSHD_CONFIG_D);
    add_template(host, "Disable password login for root", template_input(template, &path, Map::new(), None))?;
    shell(host, "Validate sshd configuration", &format!("sshd -t || {{ rm -f '{path}'; exit 1; }}"))?;
    shell(host, "Reload ssh to apply root login hardening", "systemctl reload ssh")
}

fn add_bonesremote(host: &mut DeployHost<'_>, template: &Path, version: &str) -> Result<(), DeployError> {
    for (name, path) in [
        ("Ensure BonesRemote configuration root exists", paths::BONESREMOTE_CONFIG_DIR),
        ("Ensure BonesRemote sites root exists", "/root/.config/bonesremote/sites"),
    ] {
        let mut directory = DirectoryInput::new(path);
        directory.user = Some("root".to_owned());
        directory.group = Some("root".to_owned());
        directory.mode = Some(0o700);
        host.add_stateful(Directory, directory, named(name))?;
    }
    let mut data = Map::new();
    data.insert("bonesremote_version".to_owned(), json!(version));
    host.add_stateful(
        ScriptTemplate,
        ScriptTemplateInput { source: template.to_path_buf(), arguments: Vec::new(), data },
        named("Install bonesremote binary"),
    )?;
    Ok(())
}

fn add_sudoers(host: &mut DeployHost<'_>, template: &Path) -> Result<(), DeployError> {
    let data = Map::from_iter([
        (state::DEPLOY_USER.to_owned(), json!(paths::DEPLOY_USER)),
        (
            state::BONESREMOTE_PATH.to_owned(),
            json!(Path::new(paths::USR_LOCAL_BIN).join(paths::BONESREMOTE_BINARY).display().to_string()),
        ),
    ]);
    add_template(
        host,
        "Install BonesDeploy sudoers drop-in",
        template_input(template, paths::SUDOERS_PATH, data, Some(0o440)),
    )?;
    shell(
        host,
        "Validate BonesDeploy sudoers drop-in",
        &format!("visudo -c -f '{}' >/dev/null || {{ rm -f '{}'; exit 1; }}", paths::SUDOERS_PATH, paths::SUDOERS_PATH),
    )
}

fn add_etckeeper_record(host: &mut DeployHost<'_>, template: &Path) -> Result<(), DeployError> {
    let mut data = Map::new();
    data.insert("message".to_owned(), json!("BonesInfra server setup"));
    host.add_stateful(
        ScriptTemplate,
        ScriptTemplateInput { source: template.to_path_buf(), arguments: Vec::new(), data },
        named(state::ETCKEEPER_RECORD_OPERATION),
    )?;
    Ok(())
}

fn add_template(host: &mut DeployHost<'_>, name: &str, input: TemplateInput) -> Result<(), DeployError> {
    host.add_stateful(Template, input, named(name))?;
    Ok(())
}

fn template_input(source: &Path, destination: &str, data: Map<String, Value>, mode: Option<u16>) -> TemplateInput {
    let mut input = TemplateInput::new(source, destination);
    input.data = data;
    input.mode = mode;
    input
}

fn shell(host: &mut DeployHost<'_>, name: &str, command: &str) -> Result<(), DeployError> {
    host.add_with_options(Shell, ShellCommands::from(command), named(name))?;
    Ok(())
}

fn named(name: &str) -> OperationOptions {
    OperationOptions { name: Some(name.to_owned()), ..OperationOptions::default() }
}
