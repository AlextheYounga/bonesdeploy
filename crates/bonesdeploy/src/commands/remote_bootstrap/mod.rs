use std::path::Path;

use anyhow::{Context, Result};
use serde_json::Value;

use shared::config as shared_config;
use shared::paths;

use crate::config;
use crate::ui::output;
use crate::ui::prompts;

pub mod data;

pub fn run(skip_confirm: bool, show_next: bool) -> Result<()> {
    if !skip_confirm && !prompts::confirm_remote_setup()? {
        println!("Skipped.");
        return Ok(());
    }
    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    let cfg = config::load(bones_toml)?;
    let runtime = shared_config::load_runtime(Path::new(paths::LOCAL_BONES_DIR))?;

    let ssh_user = config::bootstrap_ssh_user(&cfg);

    println!("Bootstrapping remote server...");

    let mut deploy_data = Value::Object(data::base(&cfg, &runtime.web_root));
    let host = cfg.host.clone();
    if let Value::Object(ref mut map) = deploy_data {
        map.insert(String::from(shared_config::bonesinfra_input::SSH_USER), Value::String(ssh_user));
        map.insert(String::from("host"), Value::String(host));
    }

    let json = serde_json::to_string(&deploy_data).context("Failed to serialize deploy data")?;
    bonesinfra::run_with_stdin(&["setup", "apply", "--config", paths::LOCAL_BONES_TOML], &json)?;

    println!("Remote bootstrap complete.");
    if show_next {
        println!();
        println!("{}", output::next_step("bonesdeploy remote runtime"));
    }

    Ok(())
}
