use std::fs;

use super::{TEMPLATE_SETUP_VARS_FILES, TEMPLATES, project_root};

/// Uses the project name as the default service user instead of a hardcoded value.
#[test]
fn template_service_user_defaults_to_project_name_not_applications() {
    for template in TEMPLATES {
        let content = fs::read_to_string(project_root().join(template));
        assert!(content.is_ok(), "failed to read {template}");
        let content = content.unwrap_or_default();

        assert!(
            !content.contains("service_user: 'applications'"),
            "template {template} still hardcodes applications as the service user\n{content}"
        );
    }
}

/// Defines runtime role, setup label, and apt package metadata in template vars files.
#[test]
fn template_setup_vars_files_define_runtime_and_package_metadata() {
    for vars_file in TEMPLATE_SETUP_VARS_FILES {
        let path = project_root().join(vars_file);
        let content = fs::read_to_string(&path);
        assert!(content.is_ok(), "failed to read {}", path.display());
        let content = content.unwrap_or_default();

        assert!(
            content.contains("runtime_role:"),
            "template vars file {vars_file} must define the runtime role\n{content}"
        );
        assert!(
            content.contains("setup_label:"),
            "template vars file {vars_file} must define the setup label\n{content}"
        );
        assert!(
            content.contains("setup_apt_packages:"),
            "template vars file {vars_file} must define the apt package list\n{content}"
        );
    }
}
/// Pins the Laravel PHP version in the template setup vars file so sites can override it per template.
#[test]
fn laravel_template_setup_vars_file_defines_php_version() {
    let path = project_root().join("templates/laravel/.lib/remote/vars/setup.yml");
    let content = fs::read_to_string(&path);
    assert!(content.is_ok(), "failed to read {}", path.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("laravel_php_version: \"8.3\""),
        "laravel template setup vars must define the PHP version override
{content}"
    );
}

/// Templates PHP package names in Laravel setup apt packages so they match the configured PHP version.
#[test]
fn laravel_template_setup_apt_packages_use_versioned_php_packages() {
    let path = project_root().join("templates/laravel/.lib/remote/vars/setup.yml");
    let content = fs::read_to_string(&path);
    assert!(content.is_ok(), "failed to read {}", path.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("\"php{{ laravel_php_version }}\""),
        "laravel template setup apt packages must use versioned PHP package names\n{content}"
    );
    assert!(
        content.contains("\"php{{ laravel_php_version }}-fpm\""),
        "laravel template setup apt packages must include versioned PHP-FPM\n{content}"
    );
}

/// Keeps the Laravel runtime role defaults focused on runtime layout instead of PHP version selection.
#[test]
fn laravel_runtime_role_defaults_do_not_define_php_version() {
    let path = project_root().join("templates/laravel/.lib/remote/roles/laravel_runtime/defaults/main.yml");
    let content = fs::read_to_string(&path);
    assert!(content.is_ok(), "failed to read {}", path.display());
    let content = content.unwrap_or_default();

    assert!(
        !content.contains("laravel_php_version:"),
        "laravel runtime role defaults should not hardcode PHP version selection
{content}"
    );
}

/// Defines the base apt packages in the shared scaffold vars file.
#[test]
fn shared_remote_scaffold_vars_file_defines_base_apt_packages() {
    let vars_file = project_root().join("kit/.lib/remote/vars/setup.yml");
    let content = fs::read_to_string(&vars_file);
    assert!(content.is_ok(), "failed to read {}", vars_file.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("setup_apt_packages:"),
        "shared remote vars file must define the base apt package list\n{content}"
    );
    assert!(content.contains("nginx"), "shared remote vars file must include nginx in apt packages\n{content}");
    assert!(content.contains("certbot"), "shared remote vars file must include certbot in apt packages\n{content}");
}

/// Does not install global npm packages in SPA template runtime roles.
#[test]
fn spa_template_runtime_roles_do_not_install_global_npm_packages() {
    for template in ["next", "sveltekit", "vue"] {
        let defaults =
            project_root().join(format!("templates/{template}/.lib/remote/roles/{template}_runtime/defaults/main.yml"));
        assert!(!defaults.exists(), "{template} runtime role should not define setup-time global npm packages");

        let tasks =
            project_root().join(format!("templates/{template}/.lib/remote/roles/{template}_runtime/tasks/main.yml"));
        let content = fs::read_to_string(&tasks);
        assert!(content.is_ok(), "failed to read {}", tasks.display());
        let content = content.unwrap_or_default();

        assert!(
            !content.contains("npm install -g"),
            "{template} runtime role should not install globals during setup because the project Node version is resolved later\n{content}"
        );
    }
}
