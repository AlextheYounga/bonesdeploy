use super::{Bones, IS_STATIC_KEY, Question, QuestionKind};

pub fn questions() -> &'static [Question] {
    &[Question { key: IS_STATIC_KEY, label: "Is this Nuxt site static?", kind: QuestionKind::Bool { default: true } }]
}

pub(crate) fn configure(cfg: &mut Bones) {
    let is_static = cfg.runtime.extra.get(IS_STATIC_KEY).is_some_and(|value| value.to_string() == "true");
    if is_static {
        cfg.runtime.web_root = String::from(".output/public");
    }
}

pub(crate) fn environment_example() -> String {
    super::join_env_lines(&["NODE_ENV=production", "NUXT_PUBLIC_SITE_URL=https://example.com"])
}

pub(crate) fn build_environment_example() -> String {
    super::join_env_lines(&[super::BUILD_ENV_HEADER, "NODE_VERSION=", "NUXT_PUBLIC_SITE_URL="])
}
