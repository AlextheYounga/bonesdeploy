use std::path::Path;

use anyhow::{Context, Result};
use console::style;
use serde_json::Value;

use shared::config as shared_config;
use shared::paths;

use crate::bootstrap_ssh;
use crate::config;
use crate::python;
use super::remote_data;

pub fn run() -> Result<()> {
    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    let cfg = config::load(bones_toml)?;
    let runtime = shared_config::load_runtime(Path::new(paths::LOCAL_BONES_DIR))?;

    let ssh_user = bootstrap_ssh::resolve(Some(&cfg.ssh_user));

    let mut deploy_data = Value::Object(remote_data::base(&cfg, &runtime.web_root)?);
    let host = cfg.host;
    if let Value::Object(ref mut map) = deploy_data {
        map.insert(String::from("ssh_user"), Value::String(ssh_user));
        map.insert(String::from("host"), Value::String(host));
    }

    let json = serde_json::to_string(&deploy_data).context("Failed to serialize deploy data")?;
    python::run_python_with_stdin(
        &["setup", "apply", "--config", bones_toml.to_str().unwrap_or(".bones/bones.toml")],
        &json,
    )?;

    println!("{} Remote setup complete.", style("Done!").green().bold());

    Ok(())
}
