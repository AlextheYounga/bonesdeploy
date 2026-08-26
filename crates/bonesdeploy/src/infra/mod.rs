pub mod assets;
pub mod git;
pub mod ssh;

use anyhow::{Context, Result};
use bonesdeploy_core::config::{Bones, ProvisioningRequest, RemoteDeploymentConfig};

pub fn provisioning_request(config: &Bones) -> Result<String> {
    serde_json::to_string(&ProvisioningRequest::from_bones(config)?).context("Failed to serialize provisioning request")
}

pub async fn sync_control_plane(session: &openssh::Session, config: &Bones) -> Result<()> {
    let body = format!("{}\n", serde_json::to_string_pretty(&RemoteDeploymentConfig::from_bones(config))?);
    let command = format!("bonesremote config sync --site {} --config-stdin", ssh::shell_quote(&config.project_name));
    ssh::stream_cmd_with_stdin(session, &command, body.as_bytes()).await
}
