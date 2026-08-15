use super::{FrameworkDefaults, PermissionDefault, Question, directory, file};

const PERMISSIONS: [PermissionDefault; 3] = [directory("*", 750, false), file("*", 640), directory("build", 770, true)];

pub(crate) fn defaults() -> FrameworkDefaults {
    FrameworkDefaults { template: "sveltekit", web_root: "build", language: None, permissions: &PERMISSIONS }
}

/// `SvelteKit` takes no framework configuration.
pub fn questions() -> &'static [Question] {
    &[]
}

pub(crate) fn environment_example(_project_name: &str, site_url: &str) -> String {
    super::join_env_lines(&["NODE_ENV=production", &format!("ORIGIN={site_url}")])
}

pub(crate) fn build_environment_example() -> String {
    super::join_env_lines(&[super::BUILD_ENV_HEADER])
}
