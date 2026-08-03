use super::{Question, QuestionKind};
use bonesdeploy_core::config::Framework;

const DEFAULT_PYTHON_VERSION: &str = "3.14";

pub fn questions() -> &'static [Question] {
    &[
        Question {
            key: "python_version",
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

pub(crate) fn environment_example(project_name: &str, _site_url: &str) -> String {
    super::join_env_lines(&[
        &format!("DJANGO_SETTINGS_MODULE={project_name}.settings.production"),
        "SECRET_KEY=",
        &format!("DATABASE_URL=sqlite:////srv/sites/{project_name}/shared/database.sqlite"),
    ])
}

pub(crate) fn build_environment_example(framework: &Framework) -> String {
    let python_version =
        framework.extra.get("python_version").and_then(|value| value.as_str()).unwrap_or(DEFAULT_PYTHON_VERSION);
    super::join_env_lines(&[super::BUILD_ENV_HEADER, &format!("PYTHON_VERSION={python_version}")])
}
