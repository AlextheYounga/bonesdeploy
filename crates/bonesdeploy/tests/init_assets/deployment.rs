use std::fs;

use super::project_root;

/// Installs pnpm only after nvm activates the project Node version in the Nuxt deployment script.
#[test]
fn nuxt_deployment_script_installs_pnpm_after_nvm() {
    let script = project_root().join("templates/nuxt/deployment/01_run_deployment_concerns.sh");
    let content = fs::read_to_string(&script);
    assert!(content.is_ok(), "failed to read {}", script.display());
    let content = content.unwrap_or_default();

    let nvm_install = content.find("nvm install");
    let pnpm_install = content.find("npm install -g pnpm");

    assert!(
        nvm_install.is_some(),
        "Nuxt deploy script must load the project Node version before installing globals\n{content}"
    );
    assert!(
        pnpm_install.is_some(),
        "Nuxt deploy script must install pnpm under the active project Node version when pnpm is the package manager\n{content}"
    );
    assert!(
        nvm_install < pnpm_install,
        "pnpm must be installed only after nvm activates the project Node version\n{content}"
    );
    assert!(
        content.contains("npm install --include=optional"),
        "Nuxt npm installs must preserve optional native dependencies for oxc-parser\n{content}"
    );
}

/// Does not install global npm packages in the Nuxt runtime role.
#[test]
fn nuxt_runtime_role_does_not_install_global_npm_packages() {
    let defaults = project_root().join("templates/nuxt/remote/roles/nuxt_runtime/defaults/main.yml");
    assert!(
        !defaults.exists(),
        "Nuxt runtime role should not define setup-time global npm packages"
    );

    let tasks = project_root().join("templates/nuxt/remote/roles/nuxt_runtime/tasks/main.yml");
    let content = fs::read_to_string(&tasks);
    assert!(content.is_ok(), "failed to read {}", tasks.display());
    let content = content.unwrap_or_default();

    assert!(
        !content.contains("npm install -g"),
        "Nuxt runtime role should not install globals during setup because the project Node version is resolved later\n{content}"
    );
}

/// Does not use pm2 in SPA template deployment scripts.
#[test]
fn spa_template_deploy_scripts_do_not_use_pm2() {
    for template in ["nuxt", "next", "sveltekit", "vue"] {
        let script = project_root()
            .join(format!("templates/{template}/deployment/01_run_deployment_concerns.sh"));
        let content = fs::read_to_string(&script);
        assert!(content.is_ok(), "failed to read {}", script.display());
        let content = content.unwrap_or_default();

        assert!(
            !content.contains("pm2"),
            "{template} deployment script should not use pm2 — process lifecycle is managed by bonesremote via systemd"
        );
    }
}

/// Does not use unsafe `PROJECT_NAME` fallback patterns in any deployment script.
#[test]
fn deployment_scripts_have_no_unsafe_project_name_fallbacks() {
    let patterns = [
        "${PROJECT_NAME:-",
        "${PROJECT_NAME-",
        ":-${PROJECT_NAME}",
        ":-${PROJECT_NAME:-",
    ];
    for template in ["nuxt", "next", "sveltekit", "vue", "rails", "django", "laravel"] {
        let script = project_root()
            .join(format!("templates/{template}/deployment/01_run_deployment_concerns.sh"));
        let content = fs::read_to_string(&script);
        assert!(content.is_ok(), "failed to read {}", script.display());
        let content = content.unwrap_or_default();

        for pattern in patterns {
            assert!(
                !content.contains(pattern),
                "{template} deployment script should not use unsafe PROJECT_NAME fallback '{pattern}' — project identity must be explicit"
            );
        }
    }
}

/// Restarts the project systemd service from app server deployment scripts.
#[test]
fn app_server_templates_restart_project_service_from_deployment_scripts() {
    for template in ["rails", "django"] {
        let script = project_root()
            .join(format!("templates/{template}/deployment/01_run_deployment_concerns.sh"));
        let content = fs::read_to_string(&script);
        assert!(content.is_ok(), "failed to read {}", script.display());
        let content = content.unwrap_or_default();

        assert!(
            content.contains("SERVICE_NAME=\"$PROJECT_NAME\""),
            "{template} deployment script must restart the configured project service without fallback\n{content}"
        );
        assert!(
            content.contains("systemctl restart \"$SERVICE_NAME\""),
            "{template} deployment script must restart its app server after deploy\n{content}"
        );
    }
}

/// Ends after the build step without process restart for the static Vue SPA.
#[test]
fn vue_deployment_script_ends_after_build() {
    let script = project_root().join("templates/vue/deployment/01_run_deployment_concerns.sh");
    let content = fs::read_to_string(&script);
    assert!(content.is_ok(), "failed to read {}", script.display());
    let content = content.unwrap_or_default();

    assert!(
        !content.contains("systemctl") && !content.contains("pm2") && !content.contains("restart"),
        "Vue deployment script should only build static files — no process restart needed for static SPA"
    );
}
