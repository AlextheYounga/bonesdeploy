use super::{FrameworkDefaults, PermissionDefault, Question, QuestionKind, directory, file};

const PERMISSIONS: [PermissionDefault; 3] = [directory("*", 750, false), file("*", 640), directory("media", 770, true)];

pub(super) fn defaults() -> FrameworkDefaults {
    FrameworkDefaults { template: "django", web_root: "public", language: None, permissions: &PERMISSIONS }
}

pub(super) fn questions() -> &'static [Question] {
    &[Question {
        key: "wsgi_module",
        label: "WSGI module",
        kind: QuestionKind::Text { default: "config.wsgi:application" },
    }]
}

pub(super) fn environment_example(project_name: &str, _site_url: &str) -> String {
    super::render_env_template(
        include_str!("../../assets/frameworks/django/django.env.example"),
        &[("{project_name}", project_name)],
    )
}

pub(super) fn build_environment_example() -> String {
    include_str!("../../assets/frameworks/django/django.env.build.example").to_string()
}
