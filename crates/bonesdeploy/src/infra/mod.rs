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
    format!("sudo -n bonesremote deploy --site {}", ssh::shell_quote(site))
}

pub fn artifact_deploy_command(site: &str) -> String {
    format!("sudo -n bonesremote deploy --site {} --artifact-stdin", ssh::shell_quote(site))
}

pub fn sync_control_plane_command(site: &str) -> String {
    format!("sudo -n bonesremote config sync --site {}", ssh::shell_quote(site))
}

pub fn decommission_command(action: &str, site: &str) -> String {
    format!("sudo -n bonesremote decommission {action} --site {}", ssh::shell_quote(site))
}

#[cfg(test)]
mod tests {
    use super::{artifact_deploy_command, decommission_command, deploy_command, sync_control_plane_command};

    #[test]
    fn deploy_command_runs_the_no_flag_lifecycle_through_sudo() {
        let command = deploy_command("demo");

        assert_eq!(command, "sudo -n bonesremote deploy --site 'demo'");
        assert!(!command.contains("config-stdin"));
        assert!(!command.contains("--revision"));
    }

    #[test]
    fn deploy_command_quotes_the_site() {
        assert_eq!(deploy_command("site name"), "sudo -n bonesremote deploy --site 'site name'");
    }

    #[test]
    fn artifact_deploy_command_uses_the_narrow_stdin_form() {
        assert_eq!(artifact_deploy_command("demo"), "sudo -n bonesremote deploy --site 'demo' --artifact-stdin");
    }

    #[test]
    fn sync_command_is_the_other_sudoed_deploy_command() {
        assert_eq!(sync_control_plane_command("demo"), "sudo -n bonesremote config sync --site 'demo'");
    }

    #[test]
    fn decommission_command_runs_a_sudoed_remote_transition() {
        assert_eq!(decommission_command("begin", "demo"), "sudo -n bonesremote decommission begin --site 'demo'");
    }
}
