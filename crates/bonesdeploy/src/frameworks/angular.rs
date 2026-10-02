use super::{FrameworkDefaults, PermissionDefault, Question, directory, file};

const PERMISSIONS: [PermissionDefault; 3] = [directory("*", 750, false), file("*", 640), directory("dist", 755, true)];

pub(super) fn defaults() -> FrameworkDefaults {
    FrameworkDefaults { template: "angular", web_root: "dist/browser", language: None, permissions: &PERMISSIONS }
}

/// `Angular` takes no framework configuration.
pub(super) fn questions() -> &'static [Question] {
    &[]
}

pub(super) fn environment_example(_project_name: &str, _site_url: &str) -> String {
    include_str!("../../assets/frameworks/angular/angular.env.example").to_string()
}

pub(super) fn build_environment_example() -> String {
    include_str!("../../assets/frameworks/angular/angular.env.build.example").to_string()
}
