use super::{Question, QuestionKind};

pub fn questions() -> &'static [Question] {
    &[Question {
        key: "php_version",
        label: "PHP version",
        kind: QuestionKind::Choice { choices: &["8.2", "8.3", "8.4", "8.5"], default: "8.5" },
    }]
}
