use super::{IS_STATIC_KEY, Question, QuestionKind};

pub fn questions() -> &'static [Question] {
    &[Question { key: IS_STATIC_KEY, label: "Is this Nuxt site static?", kind: QuestionKind::Bool { default: true } }]
}
