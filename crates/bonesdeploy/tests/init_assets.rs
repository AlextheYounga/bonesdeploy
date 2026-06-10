use std::fs;
use std::path::Path;

const TEMPLATES: [&str; 7] = [
    "templates/django/bones.yaml",
    "templates/laravel/bones.yaml",
    "templates/next/bones.yaml",
    "templates/nuxt/bones.yaml",
    "templates/rails/bones.yaml",
    "templates/sveltekit/bones.yaml",
    "templates/vue/bones.yaml",
];

const TEMPLATE_SETUP_VARS_FILES: [&str; 7] = [
    "templates/django/remote/vars/setup.yml",
    "templates/laravel/remote/vars/setup.yml",
    "templates/next/remote/vars/setup.yml",
    "templates/nuxt/remote/vars/setup.yml",
    "templates/rails/remote/vars/setup.yml",
    "templates/sveltekit/remote/vars/setup.yml",
    "templates/vue/remote/vars/setup.yml",
];

const TEMPLATE_SETUP_PLAYBOOKS: [&str; 7] = [
    "templates/django/remote/playbooks/setup.yml",
    "templates/laravel/remote/playbooks/setup.yml",
    "templates/next/remote/playbooks/setup.yml",
    "templates/nuxt/remote/playbooks/setup.yml",
    "templates/rails/remote/playbooks/setup.yml",
    "templates/sveltekit/remote/playbooks/setup.yml",
    "templates/vue/remote/playbooks/setup.yml",
];

/// Uses the project name as the default service user instead of a hardcoded value.
#[test]
fn template_service_user_defaults_to_project_name_not_applications() {
    for template in TEMPLATES {
        let content = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join(template));
        assert!(content.is_ok(), "failed to read {template}");
        let content = content.unwrap_or_default();

        assert!(
            !content.contains("service_user: 'applications'"),
            "template {template} still hardcodes applications as the service user\n{content}"
        );
    }
}

/// Includes the `AppArmor` role in the shared remote setup playbook.
#[test]
fn remote_setup_playbook_includes_apparmor_role() {
    let playbook = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/playbooks/setup.yml");
    let content = fs::read_to_string(&playbook);
    assert!(content.is_ok(), "failed to read {}", playbook.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("name: apparmor") && content.contains("include_role"),
        "remote setup playbook must include apparmor through shared role composition\n{content}"
    );
}

/// Loads shared template variables and includes the doctor validation task in the remote setup playbook.
#[test]
fn remote_setup_playbook_loads_shared_template_vars_and_doctor_task() {
    let playbook = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/playbooks/setup.yml");
    let content = fs::read_to_string(&playbook);
    assert!(content.is_ok(), "failed to read {}", playbook.display());
    let content = content.unwrap_or_default();

    assert!(content.contains("vars_files:"), "remote setup playbook should load shared template vars\n{content}");
    assert!(
        content.contains("../vars/setup.yml"),
        "remote setup playbook should load template override vars from a dedicated file\n{content}"
    );
    assert!(
        content.contains("include_role:"),
        "remote setup playbook should compose runtime roles from shared logic\n{content}"
    );
    assert!(
        content.contains("Run bonesremote doctor as deploy user"),
        "remote setup playbook must keep the doctor validation in the shared playbook\n{content}"
    );
}

/// Sets an `AppArmor` profile in the per-site nginx systemd service template.
#[test]
fn nginx_service_template_sets_apparmor_profile() {
    let service_template =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/nginx/site-nginx.service.j2");
    let content = fs::read_to_string(&service_template);
    assert!(content.is_ok(), "failed to read {}", service_template.display());
    let content = content.unwrap_or_default();

    assert!(content.contains("AppArmorProfile="), "per-site systemd service must pin an AppArmor profile\n{content}");
}

/// Requires the `AppArmor` service in the nginx systemd service template.
#[test]
fn nginx_service_template_waits_for_apparmor_service() {
    let service_template =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/nginx/site-nginx.service.j2");
    let content = fs::read_to_string(&service_template);
    assert!(content.is_ok(), "failed to read {}", service_template.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("After=network.target apparmor.service"),
        "per-site systemd service must start after apparmor.service\n{content}"
    );
    assert!(
        content.contains("Requires=apparmor.service"),
        "per-site systemd service must require apparmor.service\n{content}"
    );
}

/// Ensures the `AppArmor` profile template file exists at the expected path.
#[test]
fn apparmor_profile_template_exists() {
    let profile_template =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/apparmor/project-nginx-profile.j2");
    assert!(profile_template.exists(), "expected AppArmor profile template at {}", profile_template.display());
}

/// Allows reading the repo-level nginx configuration in the `AppArmor` profile template.
#[test]
fn apparmor_profile_template_allows_repo_nginx_conf() {
    let profile_template =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/apparmor/project-nginx-profile.j2");
    let content = fs::read_to_string(&profile_template);
    assert!(content.is_ok(), "failed to read {}", profile_template.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("{{ paths.repo_nginx_config }} r,"),
        "AppArmor template must allow reading repo-local nginx.conf used by bonesremote landlock nginx\n{content}"
    );
}

/// Does not deny the parent home path when the repo path is derived from the shared helper.
#[test]
fn apparmor_profile_template_does_not_deny_repo_path_parent_home() {
    let profile_template =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/apparmor/project-nginx-profile.j2");
    let content = fs::read_to_string(&profile_template);
    assert!(content.is_ok(), "failed to read {}", profile_template.display());
    let content = content.unwrap_or_default();

    assert!(
        !content.contains("deny /home/** r,"),
        "AppArmor template must not deny all /home reads because default repo_path is derived from the shared helper\n{content}"
    );
    assert!(
        !content.contains("deny /home/{{ deploy_user }}/** r,"),
        "AppArmor template must not deny deploy user home globally because repo_path defaults under that path\n{content}"
    );
}

/// Limits network access to unix stream sockets and denies inet sockets.
#[test]
fn apparmor_profile_template_limits_network_to_unix_stream() {
    let profile_template =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/apparmor/project-nginx-profile.j2");
    let content = fs::read_to_string(&profile_template);
    assert!(content.is_ok(), "failed to read {}", profile_template.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("network unix stream,"),
        "AppArmor template must permit unix stream sockets for per-site nginx\n{content}"
    );
    assert!(
        !content.contains("network inet stream,"),
        "AppArmor template should not permit inet stream by default for unix-socket based per-site nginx\n{content}"
    );
    assert!(
        !content.contains("network inet6 stream,"),
        "AppArmor template should not permit inet6 stream by default for unix-socket based per-site nginx\n{content}"
    );
}

/// Verifies template-specific playbooks are removed in favor of shared kit setup logic.
#[test]
fn template_playbooks_include_apparmor_role() {
    for playbook in TEMPLATE_SETUP_PLAYBOOKS {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join(playbook);
        assert!(!path.exists(), "template playbook {playbook} should be removed in favor of shared kit setup logic");
    }
}

/// Defines runtime role and setup label metadata in template vars files.
#[test]
fn template_setup_vars_files_define_runtime_metadata() {
    for vars_file in TEMPLATE_SETUP_VARS_FILES {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join(vars_file);
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

/// Installs pnpm only after nvm activates the project Node version in the Nuxt deployment script.
#[test]
fn nuxt_deployment_script_installs_pnpm_after_nvm() {
    let script = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("templates/nuxt/deployment/01_run_deployment_concerns.sh");
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
    let defaults = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("templates/nuxt/remote/roles/nuxt_runtime/defaults/main.yml");
    assert!(!defaults.exists(), "Nuxt runtime role should not define setup-time global npm packages");

    let tasks = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("templates/nuxt/remote/roles/nuxt_runtime/tasks/main.yml");
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
        let script = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
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
    let patterns = ["${PROJECT_NAME:-", "${PROJECT_NAME-", ":-${PROJECT_NAME}", ":-${PROJECT_NAME:-"];
    for template in ["nuxt", "next", "sveltekit", "vue", "rails", "django", "laravel"] {
        let script = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
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
        let script = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
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
    let script = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("templates/vue/deployment/01_run_deployment_concerns.sh");
    let content = fs::read_to_string(&script);
    assert!(content.is_ok(), "failed to read {}", script.display());
    let content = content.unwrap_or_default();

    assert!(
        !content.contains("systemctl") && !content.contains("pm2") && !content.contains("restart"),
        "Vue deployment script should only build static files — no process restart needed for static SPA"
    );
}

/// Does not install global npm packages in SPA template runtime roles.
#[test]
fn spa_template_runtime_roles_do_not_install_global_npm_packages() {
    for template in ["next", "sveltekit", "vue"] {
        let defaults = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(format!("templates/{template}/remote/roles/{template}_runtime/defaults/main.yml"));
        assert!(!defaults.exists(), "{template} runtime role should not define setup-time global npm packages");

        let tasks = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(format!("templates/{template}/remote/roles/{template}_runtime/tasks/main.yml"));
        let content = fs::read_to_string(&tasks);
        assert!(content.is_ok(), "failed to read {}", tasks.display());
        let content = content.unwrap_or_default();

        assert!(
            !content.contains("npm install -g"),
            "{template} runtime role should not install globals during setup because the project Node version is resolved later\n{content}"
        );
    }
}

/// Applies the `AppArmor` role before the nginx role in the shared setup playbook.
#[test]
fn shared_setup_playbook_applies_apparmor_before_nginx_role() {
    let playbook = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/playbooks/setup.yml");
    let content = fs::read_to_string(&playbook);
    assert!(content.is_ok(), "failed to read {}", playbook.display());
    let content = content.unwrap_or_default();

    let apparmor_idx = content.find("Run AppArmor role");
    let nginx_idx = content.find("Run nginx role");

    assert!(apparmor_idx.is_some(), "shared setup playbook must include apparmor include_role\n{content}");
    assert!(nginx_idx.is_some(), "shared setup playbook must include nginx include_role\n{content}");
    assert!(apparmor_idx < nginx_idx, "shared setup playbook must apply apparmor before nginx\n{content}");

    let common_idx = content.find("Run common role");
    let runtime_idx = content.find("Run runtime role");

    assert!(common_idx.is_some(), "shared setup playbook must include common role\n{content}");
    assert!(runtime_idx.is_some(), "shared setup playbook must include optional runtime role hook\n{content}");
    assert!(common_idx < runtime_idx, "runtime role hook must run after common role\n{content}");
    assert!(runtime_idx < nginx_idx, "runtime role hook must run before nginx role\n{content}");
}

/// Exposes common role defaults publicly for later runtime roles.
#[test]
fn shared_setup_playbook_exposes_common_role_defaults_to_runtime_role() {
    let playbook = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/playbooks/setup.yml");
    let content = fs::read_to_string(&playbook);
    assert!(content.is_ok(), "failed to read {}", playbook.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("name: Run common role") && content.contains("public: true"),
        "shared setup playbook must expose common role defaults for later runtime roles like nvm_dir\n{content}"
    );
}

/// Exposes nginx role defaults publicly for the later SSL role.
#[test]
fn shared_setup_playbook_exposes_nginx_role_defaults_to_ssl_role() {
    let playbook = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/playbooks/setup.yml");
    let content = fs::read_to_string(&playbook);
    assert!(content.is_ok(), "failed to read {}", playbook.display());
    let content = content.unwrap_or_default();

    let nginx_role =
        "- name: Run nginx role\n      ansible.builtin.include_role:\n        name: nginx\n        public: true";

    assert!(
        content.contains(nginx_role),
        "shared setup playbook must expose nginx defaults used by the later ssl role\n{content}"
    );
}

/// Uses resolved placeholder web root paths in the common role.
#[test]
fn shared_setup_playbook_uses_placeholder_web_root_paths() {
    let playbook = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/roles/common/tasks/main.yml");
    let content = fs::read_to_string(&playbook);
    assert!(content.is_ok(), "failed to read {}", playbook.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("{{ paths.placeholder_web_root }}"),
        "common role must seed placeholder release using the resolved placeholder web root\n{content}"
    );
    assert!(
        content.contains("{{ paths.placeholder_index }}"),
        "common role must write placeholder index through the resolved path manifest\n{content}"
    );
}

/// Uses the resolved current web root for certbot validation in the SSL role.
#[test]
fn ssl_role_uses_current_web_root_path_manifest() {
    let role = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/roles/ssl/tasks/main.yml");
    let content = fs::read_to_string(&role);
    assert!(content.is_ok(), "failed to read {}", role.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("{{ paths.current_web_root }}"),
        "ssl role must use the resolved current web root for certbot webroot validation\n{content}"
    );
}

/// Uses resolved paths in both nginx site and `AppArmor` templates.
#[test]
fn nginx_and_apparmor_templates_use_resolved_paths() {
    let nginx_site = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/nginx/site.conf.j2");
    let nginx_conf = fs::read_to_string(&nginx_site);
    assert!(nginx_conf.is_ok(), "failed to read {}", nginx_site.display());
    let nginx_conf = nginx_conf.unwrap_or_default();

    assert!(
        nginx_conf.contains("{{ paths.current_web_root }}"),
        "nginx site template must use resolved current web root\n{nginx_conf}"
    );

    let apparmor =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/apparmor/project-nginx-profile.j2");
    let apparmor_conf = fs::read_to_string(&apparmor);
    assert!(apparmor_conf.is_ok(), "failed to read {}", apparmor.display());
    let apparmor_conf = apparmor_conf.unwrap_or_default();

    assert!(
        apparmor_conf.contains("{{ paths.current_web_root }}/** r,"),
        "AppArmor profile must read the resolved current web root path\n{apparmor_conf}"
    );
    assert!(
        apparmor_conf.contains("{{ paths.repo_bones_yaml }} r,"),
        "AppArmor profile must read the resolved repo bones yaml path\n{apparmor_conf}"
    );
    assert!(
        apparmor_conf.contains("{{ paths.repo_nginx_config }} r,"),
        "AppArmor profile must read the resolved repo nginx config path\n{apparmor_conf}"
    );
}

/// Treats SSL enabled as an explicit boolean rather than relying on string truthiness.
#[test]
fn ssl_role_treats_ssl_enabled_as_explicit_boolean() {
    let role = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/roles/ssl/tasks/main.yml");
    let content = fs::read_to_string(&role);
    assert!(content.is_ok(), "failed to read {}", role.display());
    let content = content.unwrap_or_default();

    assert!(!content.contains("when: ssl_enabled\n"), "ssl role must not rely on string truthiness\n{content}");
    assert!(content.contains("when: ssl_enabled | bool"), "ssl role should cast CLI vars explicitly\n{content}");
}

/// Ensures all expected `AppArmor` role asset files exist.
#[test]
fn apparmor_role_assets_exist() {
    let role_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/roles/apparmor");

    for file in ["tasks/main.yml", "defaults/main.yml", "handlers/main.yml", "README.md"] {
        assert!(role_root.join(file).is_file(), "missing apparmor role {file}");
    }
}

/// Enforces the project `AppArmor` profile into enforce mode.
#[test]
fn apparmor_role_enforces_project_profile() {
    let tasks_file =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/roles/apparmor/tasks/main.yml");
    let content = fs::read_to_string(&tasks_file);
    assert!(content.is_ok(), "failed to read {}", tasks_file.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("aa-enforce"),
        "apparmor role must explicitly set project profile to enforce mode\n{content}"
    );
}

/// Verifies the `AppArmor` profile is loaded in the kernel.
#[test]
fn apparmor_role_verifies_profile_loaded() {
    let tasks_file =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/roles/apparmor/tasks/main.yml");
    let content = fs::read_to_string(&tasks_file);
    assert!(content.is_ok(), "failed to read {}", tasks_file.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("{{ paths.apparmor_profiles }}"),
        "apparmor role must check loaded profiles via kernel apparmor profile list\n{content}"
    );
    assert!(content.contains("apparmor_profile_name"), "apparmor role must verify expected profile name\n{content}");
}

/// Verifies the `AppArmor` profile is in enforce mode via kernel output.
#[test]
fn apparmor_role_verifies_profile_enforce_mode() {
    let tasks_file =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/roles/apparmor/tasks/main.yml");
    let content = fs::read_to_string(&tasks_file);
    assert!(content.is_ok(), "failed to read {}", tasks_file.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("{{ paths.apparmor_profiles }}")
            && content.contains("\\(enforce\\)")
            && content.contains("apparmor_profile_name | regex_escape"),
        "apparmor role must verify enforce mode directly from kernel AppArmor profiles output\n{content}"
    );
}

/// Verifies `AppArmor` is enabled in the kernel parameters.
#[test]
fn apparmor_role_verifies_kernel_enabled() {
    let tasks_file =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/roles/apparmor/tasks/main.yml");
    let content = fs::read_to_string(&tasks_file);
    assert!(content.is_ok(), "failed to read {}", tasks_file.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("{{ paths.apparmor_enabled_param }}"),
        "apparmor role must verify kernel apparmor enabled parameter\n{content}"
    );
    assert!(
        content.contains("in ['y', 'yes', '1']"),
        "apparmor role must assert enabled value is affirmative\n{content}"
    );
    assert!(
        content.contains("| trim | lower"),
        "apparmor role kernel-enabled assertion must trim aa parameter output before comparison\n{content}"
    );
}
