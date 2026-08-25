use super::{FrameworkDefaults, PermissionDefault, Question, QuestionKind, directory, file, non_recursive_file};
use bonesdeploy_core::config::Runtime;

const PHP_VERSION_KEY: &str = "php_version";
const DEFAULT_PHP_VERSION: &str = "8.5";

const PERMISSIONS: [PermissionDefault; 6] = [
    directory("*", 750, false),
    file("*", 640),
    directory("storage", 770, true),
    directory("bootstrap/cache", 770, true),
    directory("database", 770, false),
    non_recursive_file("database/database.sqlite", 660),
];

pub(super) fn defaults() -> FrameworkDefaults {
    FrameworkDefaults {
        template: "laravel",
        web_root: "public",
        language: Some((PHP_VERSION_KEY, DEFAULT_PHP_VERSION)),
        permissions: &PERMISSIONS,
    }
}

pub(super) fn questions() -> &'static [Question] {
    &[Question {
        key: PHP_VERSION_KEY,
        label: "PHP version",
        kind: QuestionKind::Choice { choices: &["8.2", "8.3", "8.4", "8.5"], default: DEFAULT_PHP_VERSION },
    }]
}

pub(super) fn environment_example(project_name: &str, site_url: &str) -> String {
    super::render_env_template(
        include_str!("../../assets/frameworks/laravel/laravel.env.example"),
        &[("{project_name}", project_name), ("{site_url}", site_url)],
    )
}

pub(super) fn build_environment_example(runtime: &Runtime) -> String {
    let php_version =
        runtime.extra.get(PHP_VERSION_KEY).and_then(|value| value.as_str()).unwrap_or(DEFAULT_PHP_VERSION);
    super::render_env_template(
        include_str!("../../assets/frameworks/laravel/laravel.env.build.example"),
        &[("{php_version}", php_version)],
    )
}
