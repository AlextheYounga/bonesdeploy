use super::Question;

/// `SvelteKit` takes no runtime configuration today.
pub fn questions() -> &'static [Question] {
    &[]
}

pub(crate) fn environment_example(_project_name: &str) -> String {
    super::join_env_lines(&["NODE_ENV=production", "ORIGIN=https://example.com"])
}

pub(crate) fn build_environment_example() -> String {
    super::join_env_lines(&[super::BUILD_ENV_HEADER, "NODE_VERSION="])
}
