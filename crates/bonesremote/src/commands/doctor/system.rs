use bonesdeploy_core::names;

use bonesdeploy_core::paths;
use std::collections::BTreeMap;
use std::env;
use std::fs;

const SUPPORTED_HOST_MESSAGE: &str = "production hosts must be Debian 12+ or Ubuntu 24.04+ on x86_64";

pub(super) fn check_supported_distribution(issues: &mut Vec<String>) {
    let os_release = fs::read_to_string(paths::ETC_OS_RELEASE);
    let Ok(os_release) = os_release else {
        issues.push(format!("Failed to read {}; {SUPPORTED_HOST_MESSAGE}", paths::ETC_OS_RELEASE));
        return;
    };

    if !supported_host(&os_release, env::consts::ARCH) {
        issues.push(SUPPORTED_HOST_MESSAGE.to_string());
    }
}

fn supported_host(os_release: &str, architecture: &str) -> bool {
    if architecture != names::X86_64 {
        return false;
    }

    let values = os_release
        .lines()
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| (key, value.trim_matches('"')))
        .collect::<BTreeMap<_, _>>();
    let Some(version) = values.get("VERSION_ID").and_then(|version| parse_version(version)) else {
        return false;
    };
    match values.get("ID") {
        Some(&"debian") if version.as_slice() >= [12].as_slice() => true,
        Some(&"ubuntu") if version.as_slice() >= [24, 4].as_slice() => true,
        _ => false,
    }
}

fn parse_version(value: &str) -> Option<Vec<u16>> {
    let parts = value.split('.').map(str::parse).collect::<Result<Vec<_>, _>>().ok()?;
    (!parts.is_empty()).then_some(parts)
}
