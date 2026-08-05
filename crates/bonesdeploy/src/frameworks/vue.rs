use super::Question;

/// `Vue` takes no framework configuration.
pub fn questions() -> &'static [Question] {
    &[]
}

pub(crate) fn environment_example(_project_name: &str, _site_url: &str) -> String {
    super::join_env_lines(&["NODE_ENV=production"])
}

pub(crate) fn build_environment_example() -> String {
    super::join_env_lines(&[super::BUILD_ENV_HEADER])
}
