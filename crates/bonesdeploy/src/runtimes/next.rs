use super::{Bones, IS_STATIC_KEY, Question, QuestionKind};

pub fn questions() -> &'static [Question] {
    &[Question { key: IS_STATIC_KEY, label: "Is this Next site static?", kind: QuestionKind::Bool { default: true } }]
}

pub(crate) fn configure(cfg: &mut Bones) {
    let is_static = cfg.runtime.extra.get(IS_STATIC_KEY).is_some_and(|value| value.to_string() == "true");
    if is_static {
        cfg.runtime.web_root = String::from("out");
    }
}

pub(crate) fn environment_example() -> String {
    super::join_env_lines(&[
        "NODE_ENV=production",
        "NEXT_PUBLIC_API_URL=\"https://api.example.com\"",
        "NEXT_PUBLIC_SITE_NAME=\"",
    ])
}

pub(crate) fn build_environment_example() -> String {
    super::join_env_lines(&[super::BUILD_ENV_HEADER, "NODE_VERSION=", "NEXT_PUBLIC_API_URL=", "NEXT_PUBLIC_SITE_NAME="])
}
