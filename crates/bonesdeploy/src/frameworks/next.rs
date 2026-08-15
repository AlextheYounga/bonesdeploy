use super::{Bones, FrameworkDefaults, IS_STATIC_KEY, PermissionDefault, Question, QuestionKind, directory, file};

const PERMISSIONS: [PermissionDefault; 3] = [directory("*", 750, false), file("*", 640), directory(".next", 770, true)];

pub(crate) fn defaults() -> FrameworkDefaults {
    FrameworkDefaults { template: "next", web_root: "public", language: None, permissions: &PERMISSIONS }
}

pub fn questions() -> &'static [Question] {
    &[Question { key: IS_STATIC_KEY, label: "Is this Next site static?", kind: QuestionKind::Bool { default: true } }]
}

pub(crate) fn configure(cfg: &mut Bones) {
    let is_static = cfg.runtime.extra.get(IS_STATIC_KEY).is_some_and(|value| value.to_string() == "true");
    if is_static {
        cfg.runtime.web_root = String::from("out");
    }
}

pub(crate) fn environment_example(project_name: &str, site_url: &str) -> String {
    super::join_env_lines(&[
        "NODE_ENV=production",
        &format!("NEXT_PUBLIC_API_URL=\"{site_url}\""),
        &format!("NEXT_PUBLIC_SITE_NAME=\"{project_name}\""),
    ])
}

pub(crate) fn build_environment_example() -> String {
    super::join_env_lines(&[super::BUILD_ENV_HEADER, "NEXT_PUBLIC_API_URL=", "NEXT_PUBLIC_SITE_NAME="])
}
