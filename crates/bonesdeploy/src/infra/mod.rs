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

pub async fn sync_control_plane(session: &openssh::Session, config: &Bones) -> Result<()> {
    let body = format!("{}\n", serde_json::to_string_pretty(&RemoteDeploymentConfig::from_bones(config))?);
    let command = sync_control_plane_command(&config.project_name);
    ssh::stream_cmd_with_stdin(session, &command, body.as_bytes()).await
}

pub fn deploy_command(site: &str) -> String {
    format!("bonesremote deploy --site {}", ssh::shell_quote(site))
}

pub fn sync_control_plane_command(site: &str) -> String {
    format!("sudo -n bonesremote config sync --site {} --config-stdin", ssh::shell_quote(site))
}

#[cfg(test)]
mod tests {
    use super::{deploy_command, sync_control_plane_command};

    #[test]
    fn deploy_command_does_not_accept_a_config_or_sudo() {
        let command = deploy_command("demo");

        assert_eq!(command, "bonesremote deploy --site 'demo'");
        assert!(!command.contains("sudo"));
        assert!(!command.contains("config-stdin"));
    }

    #[test]
    fn deploy_command_quotes_the_site() {
        assert_eq!(deploy_command("site name"), "bonesremote deploy --site 'site name'");
    }

    #[test]
    fn sync_command_is_the_only_sudoed_deploy_command() {
        assert_eq!(sync_control_plane_command("demo"), "sudo -n bonesremote config sync --site 'demo' --config-stdin");
    }
}
