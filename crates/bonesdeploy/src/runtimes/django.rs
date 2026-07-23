use super::{Question, QuestionKind};

pub fn questions() -> &'static [Question] {
    &[Question {
        key: "wsgi_module",
        label: "WSGI module",
        kind: QuestionKind::Text { default: "config.wsgi:application" },
    }]
}

pub(crate) fn environment_example(project_name: &str) -> String {
    super::join_env_lines(&[
        "DJANGO_SETTINGS_MODULE=myproject.settings.production",
        "SECRET_KEY=",
        &format!("DATABASE_URL=sqlite:////srv/sites/{project_name}/shared/database.sqlite"),
    ])
}

pub(crate) fn build_environment_example() -> String {
    super::join_env_lines(&[
        super::BUILD_ENV_HEADER,
        "# Pin Node when this project includes a frontend build.",
        "NODE_VERSION=",
    ])
}
