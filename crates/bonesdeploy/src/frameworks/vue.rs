use super::{FrameworkDefaults, PermissionDefault, Question, directory, file};

const PERMISSIONS: [PermissionDefault; 3] = [directory("*", 750, false), file("*", 640), directory("dist", 755, true)];

pub(crate) fn defaults() -> FrameworkDefaults {
    FrameworkDefaults { template: "vue", web_root: "dist", language: None, permissions: &PERMISSIONS }
}

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
