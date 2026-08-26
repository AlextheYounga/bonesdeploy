//! Environment-variable vocabulary shared by deployment scripts and containers.

pub const PROJECT_NAME: &str = "PROJECT_NAME";
pub const PROJECT_ROOT: &str = "PROJECT_ROOT";
pub const REPO_PATH: &str = "REPO_PATH";
pub const WEB_ROOT: &str = "WEB_ROOT";
pub const SERVICE_USER: &str = "SERVICE_USER";
pub const BUILD_CACHE_DIR: &str = "BUILD_CACHE_DIR";

pub const CONTAINER_CONTROLLED: &[&str] =
    &[PROJECT_NAME, PROJECT_ROOT, REPO_PATH, WEB_ROOT, SERVICE_USER, BUILD_CACHE_DIR];
