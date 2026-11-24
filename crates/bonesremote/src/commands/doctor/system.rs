use bonesdeploy_core::paths;
use std::env;
use std::fs;

use crate::inspection::platform;

const SUPPORTED_HOST_MESSAGE: &str = "production hosts must be Debian 12+ or Ubuntu 24.04+ on x86_64";

pub(super) fn check_supported_distribution(issues: &mut Vec<String>) {
    let os_release = fs::read_to_string(paths::ETC_OS_RELEASE);
    let Ok(os_release) = os_release else {
        issues.push(format!("Failed to read {}; {SUPPORTED_HOST_MESSAGE}", paths::ETC_OS_RELEASE));
        return;
    };

    if !platform::supports_production_host(&os_release, env::consts::ARCH) {
        issues.push(SUPPORTED_HOST_MESSAGE.to_string());
    }
}
