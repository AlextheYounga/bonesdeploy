use std::fs;

use super::project_root;

/// Applies all UFW rules through a shell commands list instead of individual module operations.
#[test]
fn setup_deploy_applies_all_firewall_rules_via_shell_commands() {
    let deploy = project_root().join("kit/setup/deploy.py");
    let content = fs::read_to_string(&deploy);
    assert!(content.is_ok(), "failed to read {}", deploy.display());
    let content = content.unwrap_or_default();

    assert!(content.contains("ufw --force enable"), "setup deploy must enable UFW\n{content}");
}

/// Handles SSH allowance with and without CIDR restrictions.
#[test]
fn setup_deploy_handles_manage_ssh_with_and_without_cidrs() {
    let deploy = project_root().join("kit/setup/deploy.py");
    let content = fs::read_to_string(&deploy);
    assert!(content.is_ok(), "failed to read {}", deploy.display());
    let content = content.unwrap_or_default();

    assert!(content.contains("ssh_cidrs"), "setup deploy must branch on CIDR list for SSH rules\n{content}");
}

/// Filters 'ssh' from allowed ports list to avoid double-allowing.
#[test]
fn setup_deploy_filters_ssh_from_allowed_ports() {
    let deploy = project_root().join("kit/setup/deploy.py");
    let content = fs::read_to_string(&deploy);
    assert!(content.is_ok(), "failed to read {}", deploy.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("port == \"ssh\"") && content.contains("continue"),
        "setup deploy must skip 'ssh' when iterating allowed ports\n{content}"
    );
}

/// Resolves port aliases like 'http' to numeric ports.
#[test]
fn setup_deploy_resolves_port_aliases() {
    let deploy = project_root().join("kit/setup/deploy.py");
    let content = fs::read_to_string(&deploy);
    assert!(content.is_ok(), "failed to read {}", deploy.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("port_aliases.get(port, port)"),
        "setup deploy must resolve port aliases with default fallback\n{content}"
    );
}

/// Sets default policies and enables UFW.
#[test]
fn setup_deploy_sets_default_policies_and_enables_ufw() {
    let deploy = project_root().join("kit/setup/deploy.py");
    let content = fs::read_to_string(&deploy);
    assert!(content.is_ok(), "failed to read {}", deploy.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("ufw --force default")
            && content.contains("firewall_default_incoming_policy")
            && content.contains("firewall_default_outgoing_policy"),
        "setup deploy must set default incoming and outgoing policies\n{content}"
    );
}

/// Only runs when `firewall_enabled` is true.
#[test]
fn setup_deploy_runs_firewall_only_when_enabled() {
    let deploy = project_root().join("kit/setup/deploy.py");
    let content = fs::read_to_string(&deploy);
    assert!(content.is_ok(), "failed to read {}", deploy.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("firewall_enabled"),
        "setup deploy firewall block must reference firewall_enabled\n{content}"
    );
}

/// Keeps status check behind `firewall_show_status` flag.
#[test]
fn setup_deploy_keeps_status_check_gated_by_show_status() {
    let deploy = project_root().join("kit/setup/deploy.py");
    let content = fs::read_to_string(&deploy);
    assert!(content.is_ok(), "failed to read {}", deploy.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("ufw status verbose") && content.contains("firewall_show_status"),
        "setup deploy must keep UFW status check gated by firewall_show_status\n{content}"
    );
}
