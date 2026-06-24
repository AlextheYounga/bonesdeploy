use std::path::Path;

use anyhow::{Context, Result};
use serde_json::Value;

use shared::config as shared_config;
use shared::paths;

use super::remote_data;
use crate::config;
use crate::infra::bonesinfra_cli;
use crate::infra::bootstrap_ssh;
use crate::ui::output;

pub fn run(show_next: bool) -> Result<()> {
    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    let cfg = config::load(bones_toml)?;
    let runtime = shared_config::load_runtime(Path::new(paths::LOCAL_BONES_DIR))?;

    let ssh_user = bootstrap_ssh::resolve(Some(&cfg.ssh_user));

    println!("Bootstrapping remote server...");

    let mut deploy_data = Value::Object(remote_data::base(&cfg, &runtime.web_root)?);
    let host = cfg.host;
    if let Value::Object(ref mut map) = deploy_data {
        map.insert(String::from("ssh_user"), Value::String(ssh_user));
        map.insert(String::from("host"), Value::String(host));
    }

    let json = serde_json::to_string(&deploy_data).context("Failed to serialize deploy data")?;
    bonesinfra_cli::run_with_stdin(
        &["setup", "apply", "--config", bones_toml.to_str().unwrap_or(".bones/bones.toml")],
        &json,
    )?;

    println!("Remote bootstrap complete.");
    if show_next {
        println!();
        output::next("bonesdeploy remote runtime");
    }

    Ok(())
}
