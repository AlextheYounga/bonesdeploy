use super::{Bones, FrameworkDefaults, IS_STATIC_KEY, PermissionDefault, Question, QuestionKind, directory, file};

const NUXT_STATIC_WEB_ROOT: &str = ".output/public";
const PERMISSIONS: [PermissionDefault; 4] =
    [directory("*", 750, false), file("*", 640), directory(".output", 770, true), directory(".nuxt", 770, true)];

pub(super) fn defaults() -> FrameworkDefaults {
    FrameworkDefaults { template: "nuxt", web_root: NUXT_STATIC_WEB_ROOT, language: None, permissions: &PERMISSIONS }
}

pub(super) fn questions() -> &'static [Question] {
    &[Question { key: IS_STATIC_KEY, label: "Is this Nuxt site static?", kind: QuestionKind::Bool { default: true } }]
}

pub(super) fn configure(cfg: &mut Bones) {
    let is_static = cfg.runtime.extra.get(IS_STATIC_KEY).is_some_and(|value| value.to_string() == "true");
    if is_static {
        cfg.runtime.web_root = String::from(NUXT_STATIC_WEB_ROOT);
    }
}

pub(super) fn environment_example(_project_name: &str, site_url: &str) -> String {
    super::render_env_template(
        include_str!("../../assets/frameworks/nuxt/nuxt.env.example"),
        &[("{site_url}", site_url)],
    )
}

pub(super) fn build_environment_example() -> String {
    include_str!("../../assets/frameworks/nuxt/nuxt.env.build.example").to_string()
}
