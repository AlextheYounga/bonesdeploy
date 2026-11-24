use bonesdeploy_core::state;

use std::path::Path;

use rustinfra::operations::apt::{Packages, PackagesInput};
use rustinfra::operations::files::{Download, DownloadInput};
use rustinfra::operations::server::{ScriptTemplate, ScriptTemplateInput};
use rustinfra::operations::systemd::{Service, ServiceInput};
use rustinfra::operations::{DeployError, DeployHost};
use serde_json::{Map, Value, json};

use super::{directory, ensure_nginx_dirs, framework_template, log_directories, run_as_runtime, start_application};
use crate::ProvisioningContext;
use crate::provisioning::runtime::common::{escaped, named, require_service, shell, template};

pub(super) fn add(
    host: &mut DeployHost<'_>,
    context: &ProvisioningContext,
    data: &Map<String, Value>,
    nginx: &Path,
    queue: &Path,
    php_pool: &Path,
    cleanup: &Path,
) -> Result<(), DeployError> {
    for path in ["storage", "storage/framework/sessions", "storage/framework/views", "cache", "uploads"] {
        directory(
            host,
            &format!("Ensure shared directory {path}"),
            &state::joined(&context.paths.shared, path),
            context,
            0o770,
        )?;
    }
    let version = context.extra.get(state::PHP_VERSION).and_then(Value::as_str).unwrap_or("8.5");
    let mut prerequisites = PackagesInput::new(["apt-transport-https", "ca-certificates", "curl", "lsb-release"]);
    prerequisites.update = true;
    host.add_stateful(Packages, prerequisites, named("Install PHP repo prerequisites"))?;
    let keyring_package = "/tmp/debsuryorg-archive-keyring.deb";
    host.add_stateful(
        Download,
        DownloadInput::new("https://packages.sury.org/debsuryorg-archive-keyring.deb", keyring_package),
        named("Download PHP repo keyring package"),
    )?;
    shell(host, "Install PHP repo keyring package", &format!("dpkg -i {}", escaped(keyring_package)))?;
    shell(
        host,
        "Add PHP apt repository",
        ". /etc/os-release; printf 'deb [signed-by=/usr/share/keyrings/deb.sury.org-php.gpg] https://packages.sury.org/php %s main\\n' \"$VERSION_CODENAME\" > /etc/apt/sources.list.d/php.list",
    )?;
    let packages =
        [
            "php",
            "php-cli",
            "php-fpm",
            "php-bcmath",
            "php-curl",
            "php-gd",
            "php-intl",
            "php-mbstring",
            "php-mysql",
            "php-sqlite3",
            "php-xml",
            "php-zip",
        ]
        .map(|package| {
            if package == "php" {
                format!("php{version}")
            } else {
                package.replacen("php", &format!("php{version}"), 1)
            }
        })
        .into_iter()
        .chain(["composer".to_owned()]);
    let mut install = PackagesInput::new(packages);
    install.update = true;
    host.add_stateful(Packages, install, named(&format!("Install PHP {version} runtime packages")))?;
    log_directories(host, context)?;
    let pool = state::php_pool(&context.project_name, version);
    let socket = state::php_socket(&context.project_name, version);
    let cleanup_data =
        Map::from_iter([("project".to_owned(), json!(context.project_name)), ("current_pool".to_owned(), json!(pool))]);
    host.add_stateful(
        ScriptTemplate,
        ScriptTemplateInput { source: cleanup.to_path_buf(), arguments: Vec::new(), data: cleanup_data },
        named("Remove orphaned PHP-FPM pools from other PHP versions"),
    )?;
    let mut pool_data = data.clone();
    pool_data.extend([
        ("php_fpm_pool_name".to_owned(), json!(context.project_name)),
        ("php_fpm_socket_path".to_owned(), json!(socket)),
    ]);
    template(host, "Deploy PHP-FPM pool config", php_pool, &pool, pool_data, Some(0o644))?;
    shell(host, "Validate PHP-FPM configuration", &format!("php-fpm{version} --test"))?;
    let mut fpm = ServiceInput::new(format!("php{version}-fpm"));
    fpm.enabled = Some(true);
    fpm.restarted = true;
    host.add_stateful(Service, fpm, named("Enable and restart PHP-FPM service"))?;
    let mut nginx_data = data.clone();
    nginx_data.insert("laravel_php_fpm_socket_path".to_owned(), json!(socket));
    framework_template(
        host,
        "Deploy PHP framework per-site nginx config",
        nginx,
        &context.paths.site_nginx_config,
        nginx_data,
        context,
        0o640,
    )?;
    ensure_nginx_dirs(host, context)?;
    run_as_runtime(
        host,
        context,
        "Validate nginx configuration with PHP config",
        &format!("nginx -t -c {}", escaped(&context.paths.site_nginx_config)),
    )?;
    let mut queue_data = data.clone();
    queue_data.insert("php_executable".to_owned(), json!(format!("/usr/bin/php{version}")));
    template(
        host,
        "Deploy Laravel queue worker service",
        queue,
        &context.paths.systemd_service("worker"),
        queue_data,
        Some(0o644),
    )?;
    require_service(host, context, "worker")?;
    start_application(host, context, "worker", "")
}
