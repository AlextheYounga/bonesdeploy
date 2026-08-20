use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use bonesdeploy_core::paths;

pub fn required_services(target: &str) -> Result<Vec<String>> {
    registered_services_in(target, Path::new(paths::ETC_SYSTEMD_SYSTEM))
}

pub fn registered_services_in(target: &str, systemd_root: &Path) -> Result<Vec<String>> {
    let site = target
        .strip_suffix(".target")
        .filter(|site| !site.is_empty() && !site.contains('/'))
        .with_context(|| format!("Invalid site target name: {target}"))?;
    let requires_dir = systemd_root.join(format!("{target}.requires"));
    let mut services = BTreeSet::new();

    for entry in fs::read_dir(&requires_dir).with_context(|| format!("Failed to read {}", requires_dir.display()))? {
        let entry = entry.with_context(|| format!("Failed to inspect {}", requires_dir.display()))?;
        let path = entry.path();
        let name = entry
            .file_name()
            .to_str()
            .map(str::to_owned)
            .with_context(|| format!("Invalid non-UTF-8 service registration in {}", requires_dir.display()))?;

        if !name.starts_with(&format!("{site}-")) || !name.ends_with(paths::SYSTEMD_SERVICE_SUFFIX) {
            bail!("Invalid service registration in {}: {name}", requires_dir.display());
        }
        if !fs::symlink_metadata(&path)?.file_type().is_symlink() {
            bail!("Service registration is not a symlink: {}", path.display());
        }

        let expected = systemd_root.join(&name);
        let registered = fs::canonicalize(&path)
            .with_context(|| format!("Failed to resolve service registration {}", path.display()))?;
        let expected = fs::canonicalize(&expected)
            .with_context(|| format!("Failed to resolve registered service {}", expected.display()))?;
        if registered != expected {
            bail!("Service registration points outside the site systemd units: {}", path.display());
        }
        services.insert(name);
    }

    Ok(services.into_iter().collect())
}

pub fn property(unit: &str, property: &str) -> Result<String> {
    let output = Command::new("systemctl")
        .args(["show", &format!("--property={property}"), "--value", "--no-pager", "--", unit])
        .output()
        .with_context(|| format!("Failed to inspect {property} for {unit}"))?;
    if !output.status.success() {
        bail!("Failed to inspect {property} for {unit}");
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn unit_state(unit: &str, property: &str) -> String {
    Command::new("systemctl").args([property, unit]).output().map_or_else(
        |_| String::from("unknown"),
        |output| {
            let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if value.is_empty() { String::from("unknown") } else { value }
        },
    )
}

pub fn is_active(unit: &str) -> bool {
    active_status(unit).is_ok_and(|active| active)
}

pub fn active_status(unit: &str) -> io::Result<bool> {
    Command::new("systemctl").args(["is-active", "--quiet", "--", unit]).status().map(|status| status.success())
}
