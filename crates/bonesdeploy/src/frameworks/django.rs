use super::{FrameworkDefaults, PermissionDefault, Question, QuestionKind, directory, file};
use bonesdeploy_core::config::Runtime;

const PYTHON_VERSION_KEY: &str = "python_version";
const DEFAULT_PYTHON_VERSION: &str = "3.14";

const PERMISSIONS: [PermissionDefault; 3] = [directory("*", 750, false), file("*", 640), directory("media", 770, true)];

pub(super) fn defaults() -> FrameworkDefaults {
    FrameworkDefaults {
        template: "django",
        web_root: "public",
        language: Some((PYTHON_VERSION_KEY, DEFAULT_PYTHON_VERSION)),
        permissions: &PERMISSIONS,
    }
}

pub(super) fn questions() -> &'static [Question] {
    &[
        Question {
            key: PYTHON_VERSION_KEY,
            label: "Python version",
            kind: QuestionKind::Choice { choices: &["3.12", "3.13", "3.14"], default: DEFAULT_PYTHON_VERSION },
        },
        Question {
            key: "wsgi_module",
            label: "WSGI module",
            kind: QuestionKind::Text { default: "config.wsgi:application" },
        },
    ]
}

pub(super) fn environment_example(project_name: &str, _site_url: &str) -> String {
    super::render_env_template(
        include_str!("../../assets/frameworks/django/django.env.example"),
        &[("{project_name}", project_name)],
    )
}

pub(super) fn build_environment_example(runtime: &Runtime) -> String {
    let python_version =
        runtime.extra.get(PYTHON_VERSION_KEY).and_then(|value| value.as_str()).unwrap_or(DEFAULT_PYTHON_VERSION);
    super::render_env_template(
        include_str!("../../assets/frameworks/django/django.env.build.example"),
        &[("{python_version}", python_version)],
    )
}
