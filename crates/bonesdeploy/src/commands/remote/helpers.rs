use std::path::Path;

use anyhow::{Context, Result};
use console::style;
use serde_json::Value;

use bonesdeploy_core::config as shared_config;
use bonesdeploy_core::paths;

use super::data;
use crate::config;
use crate::ui::output;
use crate::ui::prompts;

pub fn run(yes: bool) -> Result<()> {
    if !yes && !prompts::confirm_remote_helpers()? {
        println!("Skipped.");
        return Ok(());
    }

    let bones_toml = Path::new(paths::local_bones_toml());
    let cfg = config::load(bones_toml)?;
    let runtime = shared_config::load_runtime(Path::new(paths::local_bones_dir()))?;

    let ssh_user = config::bootstrap_ssh_user(&cfg);

    println!("{}", style("Installing remote helper tools").cyan().bold());

    let mut deploy_data = Value::Object(data::base(&cfg, &runtime.web_root));
    let host = cfg.host.clone();
    if let Value::Object(ref mut map) = deploy_data {
        map.insert(String::from(shared_config::bonesinfra_input::SSH_USER), Value::String(ssh_user));
        map.insert(String::from("host"), Value::String(host));
    }

    let json = serde_json::to_string(&deploy_data).context("Failed to serialize deploy data")?;
    bonesinfra::run_with_stdin(&["helpers", "apply", "--config", paths::local_bones_toml()], &json)?;

    println!("{} Helper tools installed.", output::success_marker());
    Ok(())
}
