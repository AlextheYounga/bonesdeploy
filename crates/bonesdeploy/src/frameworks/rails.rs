use super::{FrameworkDefaults, PermissionDefault, Question, QuestionKind, directory, file};

const RUBY_VERSION_KEY: &str = "ruby_version";
const DEFAULT_RUBY_VERSION: &str = "3.3";
const PERMISSIONS: [PermissionDefault; 6] = [
    directory("*", 750, false),
    file("*", 640),
    directory("tmp", 770, true),
    directory("log", 770, true),
    directory("storage", 770, true),
    directory("public/assets", 750, true),
];

pub(crate) fn defaults() -> FrameworkDefaults {
    FrameworkDefaults {
        template: "rails",
        web_root: "public",
        language: Some((RUBY_VERSION_KEY, DEFAULT_RUBY_VERSION)),
        permissions: &PERMISSIONS,
    }
}

pub fn questions() -> &'static [Question] {
    &[
        Question {
            key: RUBY_VERSION_KEY,
            label: "Ruby version",
            kind: QuestionKind::Choice { choices: &["3.2", "3.3", "3.4"], default: DEFAULT_RUBY_VERSION },
        },
        Question { key: "rails_env", label: "Rails environment", kind: QuestionKind::Text { default: "production" } },
    ]
}

pub(crate) fn environment_example(project_name: &str, _site_url: &str) -> String {
    super::join_env_lines(&[
        "RAILS_ENV=production",
        "SECRET_KEY_BASE=",
        &format!("DATABASE_URL=sqlite:////srv/sites/{project_name}/shared/storage/production.sqlite3"),
    ])
}

pub(crate) fn build_environment_example() -> String {
    super::join_env_lines(&[super::BUILD_ENV_HEADER, &format!("RUBY_VERSION={DEFAULT_RUBY_VERSION}")])
}
