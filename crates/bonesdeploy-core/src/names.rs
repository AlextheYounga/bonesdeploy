//! Shared names for configuration inputs, system resources, and managed files.

pub const APP_PROXY_TARGET: &str = "app_proxy_target";
pub const APPARMOR_PROFILE_NAME: &str = "apparmor_profile_name";
pub const BONESREMOTE_PATH: &str = "bonesremote_path";
pub const DEPLOY_USER: &str = "deploy_user";
pub const IS_STATIC: &str = "is_static";
pub const NGINX_SERVER_NAME: &str = "nginx_server_name";
pub const NGINX_SSL_CERTIFICATE_KEY_PATH: &str = "nginx_ssl_certificate_key_path";
pub const NGINX_SSL_CERTIFICATE_PATH: &str = "nginx_ssl_certificate_path";
pub const NGINX_SSL_ENABLED: &str = "nginx_ssl_enabled";
pub const NODE_VERSION: &str = "node_version";
pub const PHP_VERSION: &str = "php_version";
pub const PROJECT_NAME: &str = "project_name";
pub const RAILS_ENV: &str = "rails_env";
pub const SSH_PORT: &str = "ssh_port";
pub const SSH_USER: &str = "ssh_user";
pub const WEB_ROOT: &str = "web_root";
pub const WSGI_MODULE: &str = "wsgi_module";

pub const X86_64: &str = "x86_64";
pub const AF_UNIX: &str = "AF_UNIX";
pub const AF_UNIX_INET: &str = "AF_UNIX AF_INET";
pub const HOME: &str = "HOME";
pub const XDG_DATA_HOME: &str = "XDG_DATA_HOME";

pub const LOCAL_DATA_DIR: &str = ".local/share";
pub const NEXT_STANDALONE_DIR: &str = ".next/standalone";
pub const NUXT_PUBLIC_DIR: &str = ".output/public";
pub const NUXT_SERVER_DIR: &str = ".output/server";
pub const APT_AUTO_UPGRADES: &str = "/etc/apt/apt.conf.d/20auto-upgrades";
pub const APT_UNATTENDED_UPGRADES: &str = "/etc/apt/apt.conf.d/50unattended-upgrades";
pub const APT_KEYRINGS: &str = "/etc/apt/keyrings";
pub const NOLOGIN_SHELL: &str = "/usr/sbin/nologin";
pub const ANGULAR_BROWSER_DIR: &str = "dist/browser";

pub const ETCKEEPER_RECORD_OPERATION: &str = "Record /etc changes with etckeeper";
pub const DEFAULT_DENY_TEMPLATE: &str = "shared/nginx/default-deny.conf.j2";
pub const INDEX_TEMPLATE: &str = "shared/nginx/index.html.j2";
pub const ROUTER_TEMPLATE: &str = "shared/nginx/router.conf.j2";
pub const DEFAULT_CERTIFICATE_TEMPLATE: &str = "shared/scripts/ensure-default-deny-ssl.sh.j2";
pub const ETCKEEPER_TEMPLATE: &str = "shared/scripts/etckeeper-record.sh.j2";
pub const NGINX_VALIDATION_SCRIPT: &str = "shared/scripts/validate-nginx-safety.sh";

#[must_use]
pub fn tcp_proxy_target(port: u16) -> String {
    format!("http://127.0.0.1:{port}")
}

#[must_use]
pub fn service_unit(project: &str, name: &str) -> String {
    format!("{project}-{name}.service")
}

#[must_use]
pub fn php_pool(project: &str, version: &str) -> String {
    format!("/etc/php/{version}/fpm/pool.d/{project}.conf")
}

#[must_use]
pub fn php_socket(project: &str, version: &str) -> String {
    format!("/run/php/php{version}-fpm-{project}.sock")
}

#[must_use]
pub fn node_binary(version: &str) -> String {
    format!("/opt/bonesdeploy/node/v{version}/bin/node")
}
