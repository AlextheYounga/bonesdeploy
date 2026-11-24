use bonesdeploy_core::names;

use std::path::{Path, PathBuf};

use anyhow::Result;
use bonesdeploy_core::config::Bones;
use rustinfra::operations::apt::{Packages, PackagesInput};
use rustinfra::operations::files::{DirectoryInput, Put, PutInput};
use rustinfra::operations::server::{Script, ScriptInput, ScriptTemplate, ScriptTemplateInput};
use rustinfra::operations::{DeployError, DeployHost, PreparedDeploy};
use serde_json::{Map, json};

use super::operations::{directory, named, shell};
use crate::{ProvisioningReport, ProvisioningRuntime, ServerContext, TemplateResolver, reject_custom_provisioning};

const HELPER_PACKAGES: &[&str] = &[
    "age",
    "apt-listchanges",
    "apt-transport-https",
    "automysqlbackup",
    "autossh",
    "bash-completion",
    "bat",
    "btop",
    "fastfetch",
    "fd-find",
    "fzf",
    "gnupg",
    "iftop",
    "inotify-tools",
    "iotop",
    "jdupes",
    "jq",
    "lsb-release",
    "lsof",
    "lynis",
    "moreutils",
    "mutt",
    "ncdu",
    "powerstat",
    "powertop",
    "rename",
    "ripgrep",
    "smartmontools",
    "sysbench",
    "sysstat",
    "telnet",
    "tmux",
    "tree",
];

struct HelperAssets {
    aliases: PathBuf,
    neovim_installer: PathBuf,
    neovim_config_installer: PathBuf,
    starship_config: PathBuf,
    shell_profile: PathBuf,
    etckeeper_record: PathBuf,
}

/// Prepares the optional server-helper plan without connecting.
///
/// # Errors
/// Returns an error for missing project-local assets or an invalid operation graph.
pub fn prepare_helpers(runtime: &ProvisioningRuntime, templates: &TemplateResolver) -> Result<PreparedDeploy> {
    let assets = HelperAssets {
        aliases: templates.resolve("shared/scripts/install-debian-aliases.sh")?,
        neovim_installer: templates.resolve("shared/scripts/install-neovim.sh")?,
        neovim_config_installer: templates.resolve("shared/scripts/install-neovim-config.sh")?,
        starship_config: templates.resolve("shared/starship.toml")?,
        shell_profile: templates.resolve("shared/profile.d/bonesinfra-shell.sh")?,
        etckeeper_record: templates.resolve(names::ETCKEEPER_TEMPLATE)?,
    };
    runtime.prepare(|host| {
        add_packages(host)?;
        add_script(host, "Install Debian helper command aliases", &assets.aliases)?;
        add_starship(host, &assets)?;
        add_neovim(host, &assets)?;
        shell(
            host,
            "Install rainfrog binary",
            "curl -LSsf https://raw.githubusercontent.com/achristmascarl/rainfrog/main/install.sh | bash",
        )?;
        let mut data = Map::new();
        data.insert("message".to_owned(), json!("BonesInfra helper provisioning"));
        host.add_stateful(
            ScriptTemplate,
            ScriptTemplateInput { source: assets.etckeeper_record.clone(), arguments: Vec::new(), data },
            named(names::ETCKEEPER_RECORD_OPERATION),
        )?;
        Ok(())
    })
}

/// Installs optional server helpers directly through RustInfra.
///
/// # Errors
/// Returns an error for unsupported custom provisioning, invalid connection
/// data, missing assets, or failed operations.
pub async fn apply_helpers(config: &Bones, project_root: &Path) -> Result<ProvisioningReport> {
    reject_custom_provisioning(project_root)?;
    let server = ServerContext::from_bones(config)?;
    let runtime = ProvisioningRuntime::new(&server)?;
    let deploy = prepare_helpers(&runtime, &TemplateResolver::new(project_root))?;
    runtime.execute(&deploy).await
}

fn add_packages(host: &mut DeployHost<'_>) -> Result<(), DeployError> {
    let mut packages = PackagesInput::new(HELPER_PACKAGES.iter().copied());
    packages.update = true;
    host.add_stateful(Packages, packages, named("Install supplementary helper apt packages"))?;
    Ok(())
}

fn add_starship(host: &mut DeployHost<'_>, assets: &HelperAssets) -> Result<(), DeployError> {
    shell(
        host,
        "Install starship prompt",
        "test -x /usr/local/bin/starship || curl -fsSL https://starship.rs/install.sh | sh -s -- -y -b /usr/local/bin",
    )?;
    add_put(host, "Install starship config", root_put(&assets.starship_config, "/etc/starship.toml", 0o644))?;
    add_put(
        host,
        "Install shell profile for starship",
        root_put(&assets.shell_profile, "/etc/profile.d/bonesinfra-shell.sh", 0o644),
    )
}

fn add_neovim(host: &mut DeployHost<'_>, assets: &HelperAssets) -> Result<(), DeployError> {
    add_put(
        host,
        "Install Neovim helper script",
        root_put(&assets.neovim_installer, "/usr/local/lib/bonesinfra/install-neovim.sh", 0o755),
    )?;
    shell(host, "Install Neovim > 0.11 from official release", "/usr/local/lib/bonesinfra/install-neovim.sh")?;
    add_script(host, "Install shared Neovim config repo", &assets.neovim_config_installer)?;
    let mut permissions = DirectoryInput::new("/etc/xdg/nvim");
    permissions.user = Some("root".to_owned());
    permissions.group = Some("root".to_owned());
    permissions.mode = Some(0o755);
    directory(host, "Ensure shared Neovim config permissions", permissions)?;
    Ok(())
}

fn add_put(host: &mut DeployHost<'_>, name: &str, input: PutInput) -> Result<(), DeployError> {
    host.add_stateful(Put, input, named(name))?;
    Ok(())
}

fn root_put(source: &Path, destination: &str, mode: u16) -> PutInput {
    let mut input = PutInput::new(source, destination);
    input.user = Some("root".to_owned());
    input.group = Some("root".to_owned());
    input.mode = Some(mode);
    input
}

fn add_script(host: &mut DeployHost<'_>, name: &str, source: &Path) -> Result<(), DeployError> {
    host.add_stateful(Script, ScriptInput { source: source.to_path_buf(), arguments: Vec::new() }, named(name))?;
    Ok(())
}
