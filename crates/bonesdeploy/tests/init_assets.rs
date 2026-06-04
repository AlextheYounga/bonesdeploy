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

#[test]
fn nginx_service_template_sets_apparmor_profile() {
    let service_template =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/nginx/site-nginx.service.j2");
    let content = fs::read_to_string(&service_template);
    assert!(content.is_ok(), "failed to read {}", service_template.display());
    let content = content.unwrap_or_default();

    assert!(content.contains("AppArmorProfile="), "per-site systemd service must pin an AppArmor profile\n{content}");
}

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

#[test]
fn apparmor_profile_template_exists() {
    let profile_template =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/apparmor/project-nginx-profile.j2");
    assert!(profile_template.exists(), "expected AppArmor profile template at {}", profile_template.display());
}

#[test]
fn apparmor_profile_template_allows_repo_nginx_conf() {
    let profile_template =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/apparmor/project-nginx-profile.j2");
    let content = fs::read_to_string(&profile_template);
    assert!(content.is_ok(), "failed to read {}", profile_template.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("{{ repo_path }}/bones/nginx.conf r,"),
        "AppArmor template must allow reading repo-local nginx.conf used by bonesremote landlock nginx\n{content}"
    );
}

#[test]
fn apparmor_profile_template_does_not_deny_repo_path_parent_home() {
    let profile_template =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/apparmor/project-nginx-profile.j2");
    let content = fs::read_to_string(&profile_template);
    assert!(content.is_ok(), "failed to read {}", profile_template.display());
    let content = content.unwrap_or_default();

    assert!(
        !content.contains("deny /home/** r,"),
        "AppArmor template must not deny all /home reads because default repo_path lives under /home/git\n{content}"
    );
    assert!(
        !content.contains("deny /home/{{ deploy_user }}/** r,"),
        "AppArmor template must not deny deploy user home globally because repo_path defaults under that path\n{content}"
    );
}

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

#[test]
fn template_playbooks_include_apparmor_role() {
    for playbook in TEMPLATE_SETUP_PLAYBOOKS {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join(playbook);
        assert!(!path.exists(), "template playbook {playbook} should be removed in favor of shared kit setup logic");
    }
}

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

#[test]
fn apparmor_role_assets_exist() {
    let role_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/roles/apparmor");

    assert!(role_root.join("tasks/main.yml").is_file(), "missing apparmor role tasks/main.yml");
    assert!(role_root.join("defaults/main.yml").is_file(), "missing apparmor role defaults/main.yml");
    assert!(role_root.join("handlers/main.yml").is_file(), "missing apparmor role handlers/main.yml");
    assert!(role_root.join("README.md").is_file(), "missing apparmor role README.md");
}

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

#[test]
fn apparmor_role_verifies_profile_loaded() {
    let tasks_file =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/roles/apparmor/tasks/main.yml");
    let content = fs::read_to_string(&tasks_file);
    assert!(content.is_ok(), "failed to read {}", tasks_file.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("/sys/kernel/security/apparmor/profiles"),
        "apparmor role must check loaded profiles via kernel apparmor profile list\n{content}"
    );
    assert!(
        content.contains("apparmor_profile_name"),
        "apparmor role must verify the expected project profile name is present\n{content}"
    );
}

#[test]
fn apparmor_role_verifies_profile_enforce_mode() {
    let tasks_file =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/roles/apparmor/tasks/main.yml");
    let content = fs::read_to_string(&tasks_file);
    assert!(content.is_ok(), "failed to read {}", tasks_file.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("/sys/kernel/security/apparmor/profiles")
            && content.contains("\\(enforce\\)")
            && content.contains("apparmor_profile_name | regex_escape"),
        "apparmor role must verify enforce mode directly from kernel AppArmor profiles output\n{content}"
    );
}

#[test]
fn apparmor_role_verifies_kernel_enabled() {
    let tasks_file =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/roles/apparmor/tasks/main.yml");
    let content = fs::read_to_string(&tasks_file);
    assert!(content.is_ok(), "failed to read {}", tasks_file.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("/sys/module/apparmor/parameters/enabled"),
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
