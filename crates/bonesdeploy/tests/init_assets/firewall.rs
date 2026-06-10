use std::fs;

use super::project_root;

/// Applies all UFW rules in a single shell task instead of multiple module calls.
#[test]
fn firewall_role_applies_all_rules_in_a_single_shell_task() {
    let tasks_file = project_root().join("kit/remote/roles/firewall/tasks/main.yml");
    let content = fs::read_to_string(&tasks_file);
    assert!(content.is_ok(), "failed to read {}", tasks_file.display());
    let content = content.unwrap_or_default();

    let shell_count = content.matches("ansible.builtin.shell").count();
    let ufw_module_count = content.matches("community.general.ufw").count();

    assert_eq!(shell_count, 1, "firewall role should have exactly one shell task for rule application\n{content}");
    assert_eq!(ufw_module_count, 0, "firewall role should not use community.general.ufw module calls\n{content}");
}

/// Handles SSH allowance with and without CIDR restrictions.
#[test]
fn firewall_role_shell_handles_manage_ssh_with_and_without_cidrs() {
    let tasks_file = project_root().join("kit/remote/roles/firewall/tasks/main.yml");
    let content = fs::read_to_string(&tasks_file);
    assert!(content.is_ok(), "failed to read {}", tasks_file.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("firewall_ssh_allowed_cidrs | length"),
        "firewall role must branch on CIDR list length for SSH rules\n{content}"
    );
}

/// Filters 'ssh' from allowed ports list to avoid double-allowing.
#[test]
fn firewall_role_shell_filters_ssh_from_allowed_ports() {
    let tasks_file = project_root().join("kit/remote/roles/firewall/tasks/main.yml");
    let content = fs::read_to_string(&tasks_file);
    assert!(content.is_ok(), "failed to read {}", tasks_file.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("reject('equalto', 'ssh')"),
        "firewall role must filter 'ssh' from allowed ports list\n{content}"
    );
}

/// Resolves port aliases like 'http' to numeric ports.
#[test]
fn firewall_role_shell_resolves_port_aliases() {
    let tasks_file = project_root().join("kit/remote/roles/firewall/tasks/main.yml");
    let content = fs::read_to_string(&tasks_file);
    assert!(content.is_ok(), "failed to read {}", tasks_file.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("firewall_port_aliases[port] | default(port)"),
        "firewall role must resolve port aliases with default fallback\n{content}"
    );
}

/// Sets default policies and enables UFW.
#[test]
fn firewall_role_shell_sets_default_policies_and_enables_ufw() {
    let tasks_file = project_root().join("kit/remote/roles/firewall/tasks/main.yml");
    let content = fs::read_to_string(&tasks_file);
    assert!(content.is_ok(), "failed to read {}", tasks_file.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("ufw --force default")
            && content.contains("firewall_default_incoming_policy")
            && content.contains("firewall_default_outgoing_policy"),
        "firewall role must set default incoming and outgoing policies\n{content}"
    );
    assert!(content.contains("ufw --force enable"), "firewall role must enable UFW\n{content}");
}

/// Only runs when `firewall_enabled` is true.
#[test]
fn firewall_role_shell_runs_only_when_firewall_enabled() {
    let tasks_file = project_root().join("kit/remote/roles/firewall/tasks/main.yml");
    let content = fs::read_to_string(&tasks_file);
    assert!(content.is_ok(), "failed to read {}", tasks_file.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("when: firewall_enabled"),
        "firewall role apply task must be gated by firewall_enabled\n{content}"
    );
}

/// Keeps status check and debug output behind `show_status` flag.
#[test]
fn firewall_role_keeps_status_check_and_debug_gated_by_show_status() {
    let tasks_file = project_root().join("kit/remote/roles/firewall/tasks/main.yml");
    let content = fs::read_to_string(&tasks_file);
    assert!(content.is_ok(), "failed to read {}", tasks_file.display());
    let content = content.unwrap_or_default();

    assert!(content.contains("ufw status verbose"), "firewall role must keep status check\n{content}");
    assert!(
        content.contains("when: firewall_show_status"),
        "firewall role status tasks must be gated by firewall_show_status\n{content}"
    );
}
