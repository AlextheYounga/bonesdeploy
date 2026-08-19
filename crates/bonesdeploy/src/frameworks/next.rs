use super::{Bones, FrameworkDefaults, IS_STATIC_KEY, PermissionDefault, Question, QuestionKind, directory, file};

const PERMISSIONS: [PermissionDefault; 3] = [directory("*", 750, false), file("*", 640), directory(".next", 770, true)];

pub(super) fn defaults() -> FrameworkDefaults {
    FrameworkDefaults { template: "next", web_root: "public", language: None, permissions: &PERMISSIONS }
}

pub(super) fn questions() -> &'static [Question] {
    &[Question { key: IS_STATIC_KEY, label: "Is this Next site static?", kind: QuestionKind::Bool { default: true } }]
}

pub(super) fn configure(cfg: &mut Bones) {
    let is_static = cfg.runtime.extra.get(IS_STATIC_KEY).is_some_and(|value| value.to_string() == "true");
    if is_static {
        cfg.runtime.web_root = String::from("out");
    }
}

pub(super) fn environment_example(project_name: &str, site_url: &str) -> String {
    super::render_env_template(
        include_str!("../../assets/frameworks/next/next.env.example"),
        &[("{project_name}", project_name), ("{site_url}", site_url)],
    )
}

pub(super) fn build_environment_example() -> String {
    include_str!("../../assets/frameworks/next/next.env.build.example").to_string()
}
