use std::path::Path;

use serde_json::{Map, Value};
use shared::config as shared_config;
use shared::paths::{self, ssl_certificate_key_path, ssl_certificate_path};

use crate::config;

pub fn base(cfg: &config::Bones, web_root: &str) -> Map<String, Value> {
    let project_root = &cfg.project_root;

    let mut vars = Map::new();
    vars.insert(String::from("paths"), Value::Object(build_paths_map(cfg, project_root)));

    vars.insert(String::from(shared_config::bonesinfra_input::SSH_PORT), Value::String(cfg.port.clone()));
    vars.insert(
        String::from(shared_config::bonesinfra_input::DEPLOY_USER),
        Value::String(shared_config::default_deploy_user()),
    );
    vars.insert(
        String::from(shared_config::bonesinfra_input::RUNTIME_USER),
        Value::String(shared_config::runtime_user_for(&cfg.project_name)),
    );
    vars.insert(
        String::from(shared_config::bonesinfra_input::RUNTIME_GROUP),
        Value::String(shared_config::runtime_group_for(&cfg.project_name)),
    );
    vars.insert(
        String::from("project_root_parent"),
        Value::String(
            Path::new(project_root)
                .parent()
                .unwrap_or(Path::new(paths::DEFAULT_PROJECT_ROOT_PARENT))
                .display()
                .to_string(),
        ),
    );
    vars.insert(String::from(shared_config::bonesinfra_input::PROJECT_ROOT), Value::String(cfg.project_root.clone()));
    vars.insert(String::from("web_root"), Value::String(web_root.to_string()));
    vars.insert(String::from(shared_config::bonesinfra_input::PROJECT_NAME), Value::String(cfg.project_name.clone()));
    vars.insert(
        String::from(shared_config::bonesinfra_input::PREVIEW_DOMAIN),
        Value::String(cfg.preview_domain.clone()),
    );
    vars.insert(String::from(shared_config::bonesinfra_input::REPO_PATH), Value::String(cfg.repo_path.clone()));

    vars
}

fn build_paths_map(cfg: &config::Bones, project_root: &str) -> Map<String, Value> {
    let shared_root = Path::new(project_root).join(paths::SHARED_DIR).display().to_string();
    let releases_root = Path::new(project_root).join(paths::RELEASES_DIR).display().to_string();
    let current = Path::new(project_root).join(paths::CURRENT_LINK).display().to_string();
    let nginx_site_available =
        Path::new(paths::ETC_NGINX_SITES_AVAILABLE).join(format!("{}.conf", cfg.project_name)).display().to_string();
    let nginx_site_enabled =
        Path::new(paths::ETC_NGINX_SITES_ENABLED).join(format!("{}.conf", cfg.project_name)).display().to_string();

    let mut m = Map::new();
    m.insert(String::from("repo"), Value::String(cfg.repo_path.clone()));
    m.insert(
        String::from("repo_parent"),
        Value::String(
            Path::new(&cfg.repo_path).parent().unwrap_or(Path::new(paths::DEFAULT_REPO_PARENT)).display().to_string(),
        ),
    );
    m.insert(
        String::from("repo_head"),
        Value::String(Path::new(&cfg.repo_path).join(paths::GIT_HEAD).display().to_string()),
    );
    m.insert(
        String::from("site_nginx_config"),
        Value::String(
            Path::new(paths::DEFAULT_CONF_ROOT_PARENT)
                .join(&cfg.project_name)
                .join(paths::NGINX_CONF)
                .display()
                .to_string(),
        ),
    );
    m.insert(
        String::from("repo_deployment"),
        Value::String(
            Path::new(&cfg.repo_path).join(paths::BONES_DIR).join(paths::DEPLOYMENT_DIR).display().to_string(),
        ),
    );
    m.insert(
        String::from("conf_root"),
        Value::String(Path::new(paths::DEFAULT_CONF_ROOT_PARENT).join(&cfg.project_name).display().to_string()),
    );
    m.insert(String::from(shared_config::bonesinfra_input::PROJECT_ROOT), Value::String(project_root.to_string()));
    m.insert(
        String::from("project_root_parent"),
        Value::String(
            Path::new(project_root)
                .parent()
                .unwrap_or(Path::new(paths::DEFAULT_PROJECT_ROOT_PARENT))
                .display()
                .to_string(),
        ),
    );
    m.insert(String::from("releases"), Value::String(releases_root));
    m.insert(String::from("shared"), Value::String(shared_root));
    m.insert(String::from("current"), Value::String(current));
    m.insert(String::from("nginx_site_available"), Value::String(nginx_site_available));
    m.insert(String::from("nginx_site_enabled"), Value::String(nginx_site_enabled));
    m
}

pub fn ssl(cfg: &config::Bones, web_root: &str, domain: &str, email: &str) -> Value {
    let mut vars = base(cfg, web_root);
    vars.insert(String::from("ssl_domain"), Value::String(domain.to_string()));
    vars.insert(String::from("ssl_email"), Value::String(email.to_string()));
    vars.insert(String::from("nginx_ssl_certificate_path"), Value::String(ssl_certificate_path(domain)));
    vars.insert(String::from("nginx_ssl_certificate_key_path"), Value::String(ssl_certificate_key_path(domain)));
    Value::Object(vars)
}
