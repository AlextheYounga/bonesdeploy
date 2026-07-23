use super::{Question, QuestionKind};

pub fn questions() -> &'static [Question] {
    &[Question {
        key: "php_version",
        label: "PHP version",
        kind: QuestionKind::Choice { choices: &["8.2", "8.3", "8.4", "8.5"], default: "8.5" },
    }]
}

pub(crate) fn environment_example() -> String {
    super::join_env_lines(&[
        "APP_ENV=production",
        "APP_DEBUG=false",
        "APP_URL=https://example.com",
        "PHP_VERSION=8.5",
        "",
        "DB_CONNECTION=sqlite",
        "DB_DATABASE=/srv/sites/<project>/shared/database.sqlite",
        "",
        "LARAVEL_STORAGE_PATH=/srv/sites/<project>/shared/storage",
        "VIEW_COMPILED_PATH=/srv/sites/<project>/shared/storage/framework/views",
        "CACHE_PATH=/srv/sites/<project>/shared/cache",
        "UPLOADS_PATH=/srv/sites/<project>/shared/uploads",
    ])
}

pub(crate) fn build_environment_example() -> String {
    super::join_env_lines(&[super::BUILD_ENV_HEADER, "NODE_VERSION=", "PHP_VERSION=8.5"])
}
