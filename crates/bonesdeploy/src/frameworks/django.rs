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
    super::join_env_lines(&[
        &format!("DJANGO_SETTINGS_MODULE={project_name}.settings.production"),
        "SECRET_KEY=",
        &format!("DATABASE_URL=sqlite:////srv/sites/{project_name}/shared/database.sqlite"),
    ])
}

pub(super) fn build_environment_example(runtime: &Runtime) -> String {
    let python_version =
        runtime.extra.get(PYTHON_VERSION_KEY).and_then(|value| value.as_str()).unwrap_or(DEFAULT_PYTHON_VERSION);
    super::join_env_lines(&[super::BUILD_ENV_HEADER, &format!("PYTHON_VERSION={python_version}")])
}
