use bonesdeploy_core::config::Runtime;

use super::{FrameworkDefaults, PermissionDefault, Question, QuestionKind, directory, file};

const RUBY_VERSION_KEY: &str = "ruby_version";
const DEFAULT_RUBY_VERSION: &str = "3.3.8";
const PERMISSIONS: [PermissionDefault; 6] = [
    directory("*", 750, false),
    file("*", 640),
    directory("tmp", 770, true),
    directory("log", 770, true),
    directory("storage", 770, true),
    directory("public/assets", 750, true),
];

pub(super) fn defaults() -> FrameworkDefaults {
    FrameworkDefaults {
        template: "rails",
        web_root: "public",
        language: Some((RUBY_VERSION_KEY, DEFAULT_RUBY_VERSION)),
        permissions: &PERMISSIONS,
    }
}

pub(super) fn questions() -> &'static [Question] {
    &[
        Question {
            key: RUBY_VERSION_KEY,
            label: "Ruby version",
            kind: QuestionKind::Choice { choices: &["3.2.8", "3.3.8", "3.4.8"], default: DEFAULT_RUBY_VERSION },
        },
        Question { key: "rails_env", label: "Rails environment", kind: QuestionKind::Text { default: "production" } },
    ]
}

pub(super) fn environment_example(project_name: &str, _site_url: &str) -> String {
    super::render_env_template(
        include_str!("../../assets/frameworks/rails/rails.env.example"),
        &[("{project_name}", project_name)],
    )
}

pub(super) fn build_environment_example(runtime: &Runtime) -> String {
    let ruby_version =
        runtime.extra.get(RUBY_VERSION_KEY).and_then(|value| value.as_str()).unwrap_or(DEFAULT_RUBY_VERSION);
    super::render_env_template(
        include_str!("../../assets/frameworks/rails/rails.env.build.example"),
        &[("{ruby_version}", ruby_version)],
    )
}
