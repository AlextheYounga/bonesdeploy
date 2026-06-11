use std::fs;

use super::{TEMPLATE_SETUP_PLAYBOOKS, project_root};

/// Includes the `AppArmor` role in the shared remote setup playbook.
#[test]
fn remote_setup_playbook_includes_apparmor_role() {
    let playbook = project_root().join("kit/setup/playbooks/setup.yml");
    let content = fs::read_to_string(&playbook);
    assert!(content.is_ok(), "failed to read {}", playbook.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("name: apparmor") && content.contains("include_role"),
        "remote setup playbook must include apparmor through shared role composition\n{content}"
    );
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

/// Loads shared template variables and includes the doctor validation task in the remote setup playbook.
#[test]
fn remote_setup_playbook_loads_shared_template_vars_and_doctor_task() {
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
        content.contains("include_role:"),
        "remote setup playbook should compose runtime roles from shared logic\n{content}"
    );
    assert!(
        content.contains("Run bonesremote doctor as deploy user"),
        "remote setup playbook must keep the doctor validation in the shared playbook\n{content}"
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

/// Waits for deploy-user async jobs under the deploy user context.
#[test]
fn common_role_waits_for_deploy_user_async_jobs_as_deploy_user() {
    let tasks = project_root().join("kit/setup/roles/common/tasks/main.yml");
    let content = fs::read_to_string(&tasks);
    assert!(content.is_ok(), "failed to read {}", tasks.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("Wait for nvm install\n  become_user: \"{{ deploy_user }}\""),
        "common role must wait for nvm install as deploy_user so async status reads the correct job file\n{content}"
    );
    assert!(
        content.contains("Wait for latest LTS Node install\n  become_user: \"{{ deploy_user }}\""),
        "common role must wait for node install as deploy_user so async status reads the correct job file\n{content}"
    );
}

/// Verifies template-specific playbooks are removed in favor of shared kit setup logic.
#[test]
fn template_playbooks_include_apparmor_role() {
    for playbook in TEMPLATE_SETUP_PLAYBOOKS {
        let path = project_root().join(playbook);
        assert!(!path.exists(), "template playbook {playbook} should be removed in favor of shared kit setup logic");
    }
}

/// Applies the `AppArmor` role before the nginx role in the shared setup playbook.
#[test]
fn shared_setup_playbook_applies_apparmor_before_nginx_role() {
    let playbook = project_root().join("kit/setup/playbooks/setup.yml");
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

/// Exposes runtime role defaults publicly so later roles can use stack-specific variables.
#[test]
fn shared_setup_playbook_exposes_runtime_role_defaults_to_later_roles() {
    let playbook = project_root().join("kit/setup/playbooks/setup.yml");
    let content = fs::read_to_string(&playbook);
    assert!(content.is_ok(), "failed to read {}", playbook.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("name: \"{{ runtime_role }}\"") && content.contains("public: true"),
        "shared setup playbook must expose runtime role defaults for later roles like nginx\n{content}"
    );
}

/// Exposes common role defaults publicly for later runtime roles.
#[test]
fn shared_setup_playbook_exposes_common_role_defaults_to_runtime_role() {
    let playbook = project_root().join("kit/setup/playbooks/setup.yml");
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
    let playbook = project_root().join("kit/setup/playbooks/setup.yml");
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
