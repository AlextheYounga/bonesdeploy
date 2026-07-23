use super::{Question, QuestionKind};

pub fn questions() -> &'static [Question] {
    &[Question { key: "rails_env", label: "Rails environment", kind: QuestionKind::Text { default: "production" } }]
}

pub(crate) fn environment_example() -> String {
    super::join_env_lines(&[
        "RAILS_ENV=production",
        "SECRET_KEY_BASE=",
        "DATABASE_URL=sqlite:////srv/sites/<project>/shared/storage/production.sqlite3",
    ])
}

pub(crate) fn build_environment_example() -> String {
    super::join_env_lines(&[
        super::BUILD_ENV_HEADER,
        "# Pin Node when this project includes a frontend build.",
        "NODE_VERSION=",
    ])
}
