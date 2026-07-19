use super::{INSTALL_POSTGRES_KEY, Question, QuestionKind};

pub fn questions() -> &'static [Question] {
    &[
        Question {
            key: "wsgi_module",
            label: "WSGI module",
            kind: QuestionKind::Text { default: "config.wsgi:application" },
        },
        Question {
            key: "python_version",
            label: "Python version",
            kind: QuestionKind::Choice { choices: &["3.11", "3.12", "3.13"], default: "3.12" },
        },
        Question {
            key: INSTALL_POSTGRES_KEY,
            label: "Install PostgreSQL client libraries?",
            kind: QuestionKind::Bool { default: false },
        },
        Question { key: "static_root", label: "Static root", kind: QuestionKind::Text { default: "staticfiles" } },
        Question { key: "media_root", label: "Media root", kind: QuestionKind::Text { default: "media" } },
    ]
}
