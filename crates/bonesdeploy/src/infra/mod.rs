pub mod assets;
pub mod git;
pub mod ssh;

use anyhow::{Context, Result};
use bonesdeploy_core::config::{Bones, ProvisioningRequest, RemoteDeploymentConfig};

pub fn provisioning_request(config: &Bones) -> Result<String> {
    serde_json::to_string(&ProvisioningRequest::from_bones(config)?).context("Failed to serialize provisioning request")
}

pub fn server_request(config: &Bones) -> Result<String> {
    serde_json::to_string(&ProvisioningRequest::server_only(&config.host, &config.ssh_user, &config.port))
        .context("Failed to serialize server provisioning request")
}

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

#[cfg(test)]
mod tests {
    use super::{artifact_deploy_command, remove_site_command, sync_control_plane_command};

    #[test]
    fn artifact_deploy_command_uses_the_current_remote_cli_form() {
        assert_eq!(artifact_deploy_command("demo"), "sudo -n bonesremote deploy --site 'demo'");
    }

    #[test]
    fn sync_command_is_the_other_sudoed_deploy_command() {
        assert_eq!(sync_control_plane_command("demo"), "sudo -n bonesremote config sync --site 'demo'");
    }

    #[test]
    fn remove_site_command_runs_a_sudoed_remote_cleanup() {
        assert_eq!(remove_site_command("demo"), "sudo -n bonesremote remove-site --site 'demo'");
    }
}
