use std::collections::BTreeSet;
use std::process::Command;

use anyhow::{Context, Result, bail};
use shared::{config, paths};

use crate::privileges;

pub fn run(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote service restart")?;
    config::validate_project_name(site)?;

    let config_path = paths::bonesremote_bones_toml_path(site);
    let cfg = config::load(&config_path)
        .with_context(|| format!("Failed to load registered site state from {}", config_path.display()))?;
    if cfg.project_name != site {
        bail!("Registered site state belongs to '{}', expected '{site}'", cfg.project_name);
    }

    let target_name = paths::site_target_name(site);
    let services = target_services(&target_name)?;
    if services.is_empty() {
        bail!("Site target {target_name} has no registered services");
    }

    let status = Command::new("systemctl")
        .args(["restart", "--", &target_name])
        .status()
        .with_context(|| format!("Failed to restart {target_name}"))?;

    if !status.success() {
        bail!("Failed to restart {target_name}");
    }

    println!("Restarted {target_name}: {}", services.join(", "));
    Ok(())
}

fn target_services(target: &str) -> Result<Vec<String>> {
    let output = Command::new("systemctl")
        .args(["show", "--property=Wants", "--property=Requires", "--value", "--no-pager", "--", target])
        .output()
        .with_context(|| format!("Failed to inspect {target}"))?;
    if !output.status.success() {
        bail!("Failed to inspect {target}");
    }

    Ok(parse_target_services(&String::from_utf8_lossy(&output.stdout)))
}

fn parse_target_services(output: &str) -> Vec<String> {
    output
        .split_whitespace()
        .filter(|name| name.ends_with(paths::SYSTEMD_SERVICE_SUFFIX))
        .map(str::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::parse_target_services;

    #[test]
    fn target_dependencies_include_only_services() {
        let names = "nexttest-nginx.service nexttest-next.service nexttest.target";
        assert_eq!(parse_target_services(names), ["nexttest-next.service", "nexttest-nginx.service"]);
    }
}
