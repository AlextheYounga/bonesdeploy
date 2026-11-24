use std::collections::BTreeMap;

use bonesdeploy_core::names;

/// Returns whether an `os-release` document and architecture meet the
/// supported production-host requirements.
#[must_use]
pub fn supports_production_host(os_release: &str, architecture: &str) -> bool {
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
