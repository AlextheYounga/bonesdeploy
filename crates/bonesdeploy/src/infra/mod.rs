pub mod git;
pub mod ssh;

use anyhow::Result;
use bonesdeploy_core::config::{Bones, RemoteDeploymentConfig};

pub async fn sync_control_plane(transport: &ssh::SshTransport, config: &Bones) -> Result<()> {
    let body = format!("{}\n", serde_json::to_string_pretty(&RemoteDeploymentConfig::from_bones(config))?);
    let command = sync_control_plane_command(&config.project_name);
    transport.stream_cmd_with_stdin(&command, body.as_bytes()).await
}

pub fn artifact_deploy_command(site: &str) -> String {
    format!("sudo -n bonesremote deploy --site {}", ssh::shell_quote(site))
}

pub fn sync_control_plane_command(site: &str) -> String {
    format!("sudo -n bonesremote config sync --site {}", ssh::shell_quote(site))
}

pub fn shared_import_command(site: &str) -> String {
    format!("sudo -n bonesremote shared import --site {}", ssh::shell_quote(site))
}

pub fn shared_install_environment_command(site: &str) -> String {
    format!("sudo -n bonesremote shared install-environment --site {}", ssh::shell_quote(site))
}

pub fn remove_site_command(site: &str) -> String {
    format!("sudo -n bonesremote remove-site --site {}", ssh::shell_quote(site))
}
