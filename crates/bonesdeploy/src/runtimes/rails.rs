use super::{Question, QuestionKind};

pub fn questions() -> &'static [Question] {
    &[Question { key: "rails_env", label: "Rails environment", kind: QuestionKind::Text { default: "production" } }]
}

pub(crate) fn environment_example(project_name: &str, _site_url: &str) -> String {
    super::join_env_lines(&[
        "RAILS_ENV=production",
        "SECRET_KEY_BASE=",
        &format!("DATABASE_URL=sqlite:////srv/sites/{project_name}/shared/storage/production.sqlite3"),
    ])
}

pub(crate) fn build_environment_example() -> String {
    super::join_env_lines(&[
        super::BUILD_ENV_HEADER,
        "# Pin Node when this project includes a frontend build.",
        super::NODE_VERSION_DEFAULT,
    ])
}
