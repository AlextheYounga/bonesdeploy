use std::fs;

use super::project_root;

/// Leaves per-site AppArmor out of the shared remote setup playbook.
#[test]
fn remote_setup_playbook_excludes_apparmor_role() {
    let playbook = project_root().join("kit/setup/playbooks/setup.yml");
    let content = fs::read_to_string(&playbook);
    assert!(content.is_ok(), "failed to read {}", playbook.display());
    let content = content.unwrap_or_default();

    assert!(!content.contains("name: apparmor"), "remote setup playbook must not include per-site AppArmor\n{content}");
}

/// Includes the firewall role in the shared remote setup playbook.
#[test]
fn remote_setup_playbook_includes_firewall_role() {
    let playbook = project_root().join("kit/setup/playbooks/setup.yml");
    let content = fs::read_to_string(&playbook);
    assert!(content.is_ok(), "failed to read {}", playbook.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("name: firewall") && content.contains("include_role"),
        "remote setup playbook must include firewall through shared role composition\n{content}"
    );
}

/// Loads shared setup variables and keeps runtime validation out of the remote setup playbook.
#[test]
fn remote_setup_playbook_loads_shared_setup_vars_and_doctor_task() {
    let playbook = project_root().join("kit/setup/playbooks/setup.yml");
    let content = fs::read_to_string(&playbook);
    assert!(content.is_ok(), "failed to read {}", playbook.display());
    let content = content.unwrap_or_default();

    assert!(content.contains("vars_files:"), "remote setup playbook should load shared template vars\n{content}");
    assert!(
        content.contains("../vars/setup.yml"),
        "remote setup playbook should load template override vars from a dedicated file\n{content}"
    );
    assert!(
        !content.contains("bonesremote doctor"),
        "remote setup playbook must not run runtime doctor checks\n{content}"
    );
}

/// Uses a single shared apt package list variable in the setup playbook instead of per-role apt package definitions.
#[test]
fn shared_setup_playbook_uses_single_setup_apt_packages_manifest() {
    let playbook = project_root().join("kit/setup/playbooks/setup.yml");
    let content = fs::read_to_string(&playbook);
    assert!(content.is_ok(), "failed to read {}", playbook.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("setup_apt_packages"),
        "shared setup playbook must drive package installation from setup_apt_packages\n{content}"
    );
}

/// Starts setup apt installation before rustup bootstrap and user setup.
#[test]
fn shared_setup_playbook_starts_setup_apt_packages_before_rustup_and_users() {
    let playbook = project_root().join("kit/setup/playbooks/setup.yml");
    let content = fs::read_to_string(&playbook);
    assert!(content.is_ok(), "failed to read {}", playbook.display());
    let content = content.unwrap_or_default();

    let apt_idx = content.find("Install setup apt packages");
    let rustup_idx = content.find("Run common rustup bootstrap");
    let users_idx = content.find("Run users role");

    assert!(apt_idx.is_some(), "shared setup playbook must install setup apt packages\n{content}");
    assert!(rustup_idx.is_some(), "shared setup playbook must start rustup bootstrap\n{content}");
    assert!(users_idx.is_some(), "shared setup playbook must include users role\n{content}");
    assert!(apt_idx < rustup_idx, "setup apt packages should run before rustup bootstrap\n{content}");
    assert!(rustup_idx < users_idx, "rustup bootstrap should run before users role\n{content}");
}

/// Runs template-specific pre-package setup before installing the shared apt package manifest.
#[test]
fn shared_setup_playbook_runs_pre_package_hook_before_setup_apt_packages() {
    let playbook = project_root().join("kit/setup/playbooks/setup.yml");
    let content = fs::read_to_string(&playbook);
    assert!(content.is_ok(), "failed to read {}", playbook.display());
    let content = content.unwrap_or_default();

    let pre_packages_idx = content.find("Run template-specific pre-package setup");
    let apt_idx = content.find("Install setup apt packages");

    assert!(pre_packages_idx.is_some(), "shared setup playbook must include a pre-package hook\n{content}");
    assert!(apt_idx.is_some(), "shared setup playbook must install setup apt packages\n{content}");
    assert!(pre_packages_idx < apt_idx, "pre-package hook should run before setup apt packages\n{content}");
    assert!(
        content.contains("include_tasks: \"{{ playbook_dir }}/../tasks/pre_packages.yml\"")
            && content.contains("setup_pre_packages_enabled | default(false)"),
        "shared setup playbook must gate the pre-package hook behind template vars\n{content}"
    );
}

/// Starts the slow toolchain installers with Ansible async/poll orchestration after package installation.
#[test]
fn common_role_runs_toolchain_installers_as_async_jobs() {
    let tasks = project_root().join("kit/setup/roles/common/tasks/main.yml");
    let content = fs::read_to_string(&tasks);
    assert!(content.is_ok(), "failed to read {}", tasks.display());
    let content = content.unwrap_or_default();

    assert!(content.contains("async:"), "common role should use ansible async jobs for slow installers\n{content}");
    assert!(
        content.contains("poll: 0"),
        "common role should launch async installers without polling inline\n{content}"
    );
    assert!(
        content.contains("ansible.builtin.async_status"),
        "common role should wait on async installers with async_status\n{content}"
    );
}

/// Verifies template-specific playbooks are removed in favor of shared kit setup logic.
#[test]
fn template_playbooks_include_apparmor_role() {
    for playbook in [
        "templates/django/runtime/playbooks/setup.yml",
        "templates/laravel/runtime/playbooks/setup.yml",
        "templates/next/runtime/playbooks/setup.yml",
        "templates/nuxt/runtime/playbooks/setup.yml",
        "templates/rails/runtime/playbooks/setup.yml",
        "templates/sveltekit/runtime/playbooks/setup.yml",
        "templates/vue/runtime/playbooks/setup.yml",
    ] {
        let path = project_root().join(playbook);
        assert!(!path.exists(), "template playbook {playbook} should be removed in favor of shared kit setup logic");
    }
}

/// Leaves per-site runtime roles out of the shared setup playbook.
#[test]
fn shared_setup_playbook_keeps_runtime_roles_out() {
    let playbook = project_root().join("kit/setup/playbooks/setup.yml");
    let content = fs::read_to_string(&playbook);
    assert!(content.is_ok(), "failed to read {}", playbook.display());
    let content = content.unwrap_or_default();

    assert!(
        !content.contains("Run AppArmor role"),
        "shared setup playbook must not apply per-site AppArmor\n{content}"
    );
    assert!(!content.contains("Run runtime role"), "shared setup playbook must not apply runtime roles\n{content}");
    assert!(!content.contains("Run nginx role"), "shared setup playbook must not apply per-site nginx\n{content}");
}

/// Applies runtime, AppArmor, and nginx through the dedicated runtime playbook.
#[test]
fn remote_runtime_playbook_applies_runtime_apparmor_and_nginx() {
    let playbook = project_root().join("kit/runtime/playbooks/runtime.yml");
    let content = fs::read_to_string(&playbook);
    assert!(content.is_ok(), "failed to read {}", playbook.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("name: \"{{ runtime_role }}\"") && content.contains("public: true"),
        "runtime playbook must expose runtime role defaults for later roles like nginx\n{content}"
    );
    assert!(
        content.contains("Run AppArmor role") && content.contains("Run nginx role"),
        "runtime playbook must apply shared per-site AppArmor and nginx roles\n{content}"
    );
}

/// Installs runtime apt packages before applying runtime roles.
#[test]
fn remote_runtime_playbook_installs_runtime_packages_before_runtime_role() {
    let playbook = project_root().join("kit/runtime/playbooks/runtime.yml");
    let content = fs::read_to_string(&playbook);
    assert!(content.is_ok(), "failed to read {}", playbook.display());
    let content = content.unwrap_or_default();

    let packages_idx = content.find("Install runtime apt packages");
    let runtime_idx = content.find("Run runtime role");

    assert!(packages_idx.is_some(), "runtime playbook must install runtime apt packages\n{content}");
    assert!(runtime_idx.is_some(), "runtime playbook must apply the runtime role\n{content}");
    assert!(packages_idx < runtime_idx, "runtime packages must install before runtime role execution\n{content}");
}

/// Leaves SSL role out of the runtime playbook since SSL has its own playbook.
#[test]
fn remote_runtime_playbook_excludes_ssl_role() {
    let playbook = project_root().join("kit/runtime/playbooks/runtime.yml");
    let content = fs::read_to_string(&playbook);
    assert!(content.is_ok(), "failed to read {}", playbook.display());
    let content = content.unwrap_or_default();

    assert!(
        !content.contains("Run ssl role") && !content.contains("name: ssl"),
        "runtime playbook must not include SSL role - SSL has its own playbook\n{content}"
    );
}

/// Applies only the SSL role through the dedicated SSL playbook.
#[test]
fn remote_ssl_playbook_applies_ssl_role_only() {
    let playbook = project_root().join("kit/runtime/playbooks/ssl.yml");
    let content = fs::read_to_string(&playbook);
    assert!(content.is_ok(), "failed to read {}", playbook.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("Run ssl role") && content.contains("name: ssl"),
        "SSL playbook must apply the SSL role\n{content}"
    );
    assert!(
        !content.contains("Run AppArmor role") && !content.contains("name: apparmor"),
        "SSL playbook must not include AppArmor role\n{content}"
    );
    assert!(
        !content.contains("Run nginx role") && !content.contains("name: nginx"),
        "SSL playbook must not include general nginx role\n{content}"
    );
    assert!(
        !content.contains("Run runtime role") && !content.contains("runtime_role"),
        "SSL playbook must not include runtime role\n{content}"
    );
}
