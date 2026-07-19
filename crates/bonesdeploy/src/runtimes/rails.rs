use super::{Question, QuestionKind};

pub fn questions() -> &'static [Question] {
    &[Question { key: "rails_env", label: "Rails environment", kind: QuestionKind::Text { default: "production" } }]
}
