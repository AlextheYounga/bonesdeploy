use super::{Question, QuestionKind};

const DEFAULT_RUBY_VERSION: &str = "3.3";

pub fn questions() -> &'static [Question] {
    &[
        Question {
            key: "ruby_version",
            label: "Ruby version",
            kind: QuestionKind::Choice { choices: &["3.2", "3.3", "3.4"], default: DEFAULT_RUBY_VERSION },
        },
        Question { key: "rails_env", label: "Rails environment", kind: QuestionKind::Text { default: "production" } },
    ]
}

pub(crate) fn environment_example(project_name: &str, _site_url: &str) -> String {
    super::join_env_lines(&[
        "RAILS_ENV=production",
        "SECRET_KEY_BASE=",
        &format!("DATABASE_URL=sqlite:////srv/sites/{project_name}/shared/storage/production.sqlite3"),
    ])
}

pub(crate) fn build_environment_example() -> String {
    super::join_env_lines(&[super::BUILD_ENV_HEADER, &format!("RUBY_VERSION={DEFAULT_RUBY_VERSION}")])
}
