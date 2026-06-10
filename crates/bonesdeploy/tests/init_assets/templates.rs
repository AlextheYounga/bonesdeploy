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

/// Defines runtime role and setup label metadata in template vars files.
#[test]
fn template_setup_vars_files_define_runtime_metadata() {
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
    }
}

/// Ensures the shared scaffold embeds `kit/.lib/remote/Aptfile` as the base setup package manifest.
#[test]
fn shared_remote_scaffold_embeds_base_aptfile() {
    let aptfile = project_root().join("kit/.lib/remote/Aptfile");
    assert!(aptfile.is_file(), "expected shared remote Aptfile at {}", aptfile.display());
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
