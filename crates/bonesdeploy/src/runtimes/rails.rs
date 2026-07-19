use super::{INSTALL_POSTGRES_KEY, Question, QuestionKind};

pub fn questions() -> &'static [Question] {
    &[
        Question {
            key: "ruby_version",
            label: "Ruby version",
            kind: QuestionKind::Choice { choices: &["3.2", "3.3", "3.4"], default: "3.3" },
        },
        Question {
            key: INSTALL_POSTGRES_KEY,
            label: "Install PostgreSQL client libraries?",
            kind: QuestionKind::Bool { default: false },
        },
        Question { key: "install_redis", label: "Install Redis?", kind: QuestionKind::Bool { default: false } },
        Question { key: "rails_env", label: "Rails environment", kind: QuestionKind::Text { default: "production" } },
    ]
}
