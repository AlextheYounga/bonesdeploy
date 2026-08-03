use super::{Question, QuestionKind};
use bonesdeploy_core::config::Framework;

const PHP_DEFAULT_VERSION: &str = "8.5";

pub fn questions() -> &'static [Question] {
    &[
        Question {
            key: "php_version",
            label: "PHP version",
            kind: QuestionKind::Choice { choices: &["8.2", "8.3", "8.4", "8.5"], default: PHP_DEFAULT_VERSION },
        },
        // TODO: Set up queue worker.
        Question {
            key: "install_queue_worker",
            label: "Install Laravel queue worker?",
            kind: QuestionKind::Bool { default: false },
        },
    ]
}

#[expect(clippy::too_many_lines)]
pub(crate) fn environment_example(project_name: &str, site_url: &str) -> String {
    super::join_env_lines(&[
        "APP_NAME=Laravel",
        "APP_ENV=production",
        "APP_KEY=",
        "APP_DEBUG=true",
        &format!("APP_URL={site_url}"),
        "",
        "APP_LOCALE=en",
        "APP_FALLBACK_LOCALE=en",
        "APP_FAKER_LOCALE=en_US",
        "",
        "APP_MAINTENANCE_DRIVER=file",
        "# APP_MAINTENANCE_STORE=database",
        "",
        "# PHP_CLI_SERVER_WORKERS=4",
        "",
        "BCRYPT_ROUNDS=12",
        "",
        "LOG_CHANNEL=stack",
        "LOG_STACK=single",
        "LOG_DEPRECATIONS_CHANNEL=null",
        "LOG_LEVEL=debug",
        "",
        "DB_CONNECTION=sqlite",
        &format!("DB_DATABASE=/srv/sites/{project_name}/shared/database.sqlite"),
        "",
        &format!("LARAVEL_STORAGE_PATH=/srv/sites/{project_name}/shared/storage"),
        &format!("VIEW_COMPILED_PATH=/srv/sites/{project_name}/shared/storage/framework/views"),
        &format!("CACHE_PATH=/srv/sites/{project_name}/shared/cache"),
        &format!("UPLOADS_PATH=/srv/sites/{project_name}/shared/uploads"),
        "",
        "# DB_HOST=127.0.0.1",
        "# DB_PORT=3306",
        "# DB_DATABASE=laravel",
        "# DB_USERNAME=root",
        "# DB_PASSWORD=",
        "",
        "SESSION_DRIVER=database",
        "SESSION_LIFETIME=120",
        "SESSION_ENCRYPT=false",
        "SESSION_PATH=/",
        "SESSION_DOMAIN=null",
        "",
        "BROADCAST_CONNECTION=log",
        "FILESYSTEM_DISK=local",
        "QUEUE_CONNECTION=database",
        "",
        "CACHE_STORE=database",
        "# CACHE_PREFIX=",
        "",
        "MEMCACHED_HOST=127.0.0.1",
        "",
        "REDIS_CLIENT=phpredis",
        "REDIS_HOST=127.0.0.1",
        "REDIS_PASSWORD=null",
        "REDIS_PORT=6379",
        "",
        "MAIL_MAILER=log",
        "MAIL_SCHEME=null",
        "MAIL_HOST=127.0.0.1",
        "MAIL_PORT=2525",
        "MAIL_USERNAME=null",
        "MAIL_PASSWORD=null",
        "MAIL_FROM_ADDRESS=\"hello@example.com\"",
        "MAIL_FROM_NAME=\"${APP_NAME}\"",
        "",
        "AWS_ACCESS_KEY_ID=",
        "AWS_SECRET_ACCESS_KEY=",
        "AWS_DEFAULT_REGION=us-east-1",
        "AWS_BUCKET=",
        "AWS_USE_PATH_STYLE_ENDPOINT=false",
        "",
        "VITE_APP_NAME=\"${APP_NAME}\"",
    ])
}

pub(crate) fn build_environment_example(framework: &Framework) -> String {
    let php_version =
        framework.extra.get("php_version").and_then(|value| value.as_str()).unwrap_or(PHP_DEFAULT_VERSION);
    super::join_env_lines(&[super::BUILD_ENV_HEADER, &format!("PHP_VERSION={php_version}")])
}
