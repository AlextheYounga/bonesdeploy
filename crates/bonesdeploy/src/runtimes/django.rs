use super::{Question, QuestionKind};

pub fn questions() -> &'static [Question] {
    &[Question {
        key: "wsgi_module",
        label: "WSGI module",
        kind: QuestionKind::Text { default: "config.wsgi:application" },
    }]
}
