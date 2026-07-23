use super::{Question, QuestionKind};
use shared::config::Runtime;

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

pub(crate) fn environment_example(project_name: &str, site_url: &str) -> String {
    super::join_env_lines(&[
        "APP_ENV=production",
        "APP_DEBUG=false",
        &format!("APP_URL={site_url}"),
        "",
        "DB_CONNECTION=sqlite",
        &format!("DB_DATABASE=/srv/sites/{project_name}/shared/database.sqlite"),
        "",
        &format!("LARAVEL_STORAGE_PATH=/srv/sites/{project_name}/shared/storage"),
        &format!("VIEW_COMPILED_PATH=/srv/sites/{project_name}/shared/storage/framework/views"),
        &format!("CACHE_PATH=/srv/sites/{project_name}/shared/cache"),
        &format!("UPLOADS_PATH=/srv/sites/{project_name}/shared/uploads"),
    ])
}

pub(crate) fn build_environment_example(runtime: &Runtime) -> String {
    let php_version = runtime.extra.get("php_version").and_then(|value| value.as_str()).unwrap_or(PHP_DEFAULT_VERSION);
    super::join_env_lines(&[
        super::BUILD_ENV_HEADER,
        super::NODE_VERSION_DEFAULT,
    ])
}
