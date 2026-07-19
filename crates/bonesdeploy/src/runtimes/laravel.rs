use super::{Bones, Question, QuestionKind, set_env_value};

pub fn questions() -> &'static [Question] {
    &[
        Question {
            key: "php_version",
            label: "PHP version",
            kind: QuestionKind::Choice { choices: &["8.2", "8.3", "8.4", "8.5"], default: "8.5" },
        },
        Question {
            key: "install_queue_worker",
            label: "Install Laravel queue worker?",
            kind: QuestionKind::Bool { default: false },
        },
    ]
}

pub(crate) fn configure_env_example(content: &str, cfg: &Bones) -> String {
    if cfg.preview_domain.is_empty() {
        return content.to_string();
    }
    set_env_value(content, "APP_URL", &format!("http://{}", cfg.preview_domain))
}
