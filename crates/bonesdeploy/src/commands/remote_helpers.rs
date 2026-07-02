use std::path::Path;

use anyhow::{Context, Result};
use serde_json::Value;

use shared::config as shared_config;
use shared::paths;

use super::remote_bootstrap::data;
use crate::config;
use crate::infra::bonesinfra;
use crate::infra::bootstrap_ssh;
use crate::ui::prompts;

pub fn run(yes: bool) -> Result<()> {
    if !yes && !prompts::confirm_remote_helpers()? {
        println!("Skipped.");
        return Ok(());
    }

    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    let cfg = config::load(bones_toml)?;
    let runtime = shared_config::load_runtime(Path::new(paths::LOCAL_BONES_DIR))?;

    let ssh_user = bootstrap_ssh::resolve(Some(&cfg.ssh_user));

    println!("Installing remote helper tools...");

    let mut deploy_data = Value::Object(data::base(&cfg, &runtime.web_root));
    let host = cfg.host;
    if let Value::Object(ref mut map) = deploy_data {
        map.insert(String::from(shared_config::bonesinfra_input::SSH_USER), Value::String(ssh_user));
        map.insert(String::from("host"), Value::String(host));
    }

    let json = serde_json::to_string(&deploy_data).context("Failed to serialize deploy data")?;
    bonesinfra::run_with_stdin(&["helpers", "apply", "--config", paths::LOCAL_BONES_TOML], &json)?;

    println!("Remote helper tools installed.");
    Ok(())
}
