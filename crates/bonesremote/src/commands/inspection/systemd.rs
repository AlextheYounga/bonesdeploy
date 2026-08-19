use std::collections::BTreeSet;
use std::io;
use std::process::Command;

use anyhow::{Context, Result, bail};
use bonesdeploy_core::paths;

pub(crate) fn required_services(target: &str) -> Result<Vec<String>> {
    let output = Command::new("systemctl")
        .args(["show", "--property=Requires", "--value", "--no-pager", "--", target])
        .output()
        .with_context(|| format!("Failed to inspect {target}"))?;
    if !output.status.success() {
        bail!("Failed to inspect {target}");
    }

    Ok(parse_required_services(&String::from_utf8_lossy(&output.stdout)))
}

pub(crate) fn parse_required_services(output: &str) -> Vec<String> {
    output
        .split_whitespace()
        .filter(|name| name.ends_with(paths::SYSTEMD_SERVICE_SUFFIX))
        .map(str::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(crate) fn unit_state(unit: &str, property: &str) -> String {
    Command::new("systemctl").args([property, unit]).output().map_or_else(
        |_| String::from("unknown"),
        |output| {
            let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if value.is_empty() { String::from("unknown") } else { value }
        },
    )
}

pub(crate) fn is_active(unit: &str) -> bool {
    active_status(unit).is_ok_and(|active| active)
}

pub(crate) fn active_status(unit: &str) -> io::Result<bool> {
    Command::new("systemctl").args(["is-active", "--quiet", "--", unit]).status().map(|status| status.success())
}

#[cfg(test)]
mod tests {
    use super::parse_required_services;

    #[test]
    fn parses_unique_service_dependencies_only() {
        assert_eq!(
            parse_required_services(
                "nexttest-nginx.service nexttest-next.service nexttest.target nexttest-next.service"
            ),
            ["nexttest-next.service", "nexttest-nginx.service"]
        );
    }
}
