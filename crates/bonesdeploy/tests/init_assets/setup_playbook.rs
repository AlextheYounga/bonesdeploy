use std::fs;

use super::project_root;

/// Leaves per-site AppArmor out of the shared remote setup deploy script.
#[test]
fn remote_setup_deploy_excludes_apparmor_logic() {
    let deploy = project_root().join("infra/setup.py");
    let content = fs::read_to_string(&deploy);
    assert!(content.is_ok(), "failed to read {}", deploy.display());
    let content = content.unwrap_or_default();

    assert!(
        !content.contains("apparmor_parser") && !content.contains("apparmor_profile_name"),
        "remote setup deploy must not include per-site AppArmor\n{content}"
    );
}

/// Includes the firewall logic in the shared remote setup deploy script.
#[test]
fn remote_setup_deploy_includes_firewall_logic() {
    let deploy = project_root().join("infra/setup.py");
    let content = fs::read_to_string(&deploy);
    assert!(content.is_ok(), "failed to read {}", deploy.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("ufw --force enable"),
        "remote setup deploy must include UFW firewall configuration\n{content}"
    );
}

/// Loads shared setup variables and keeps runtime validation out of the remote setup deploy.
#[test]
fn remote_setup_deploy_keeps_runtime_checks_out() {
    let deploy = project_root().join("infra/setup.py");
    let content = fs::read_to_string(&deploy);
    assert!(content.is_ok(), "failed to read {}", deploy.display());
    let content = content.unwrap_or_default();

    assert!(
        !content.contains("bonesremote doctor"),
        "remote setup deploy must not run runtime doctor checks\n{content}"
    );
}

/// Uses a single shared apt package list in the setup deploy script.
#[test]
fn shared_setup_deploy_uses_single_setup_apt_packages_manifest() {
    let deploy = project_root().join("infra/setup.py");
    let content = fs::read_to_string(&deploy);
    assert!(content.is_ok(), "failed to read {}", deploy.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("SETUP_APT_PACKAGES"),
        "shared setup deploy must drive package installation from a single manifest\n{content}"
    );
}

/// Starts setup apt installation before rustup bootstrap and user creation.
#[test]
fn shared_setup_deploy_starts_packages_before_users_and_rustup() {
    let deploy = project_root().join("infra/setup.py");
    let content = fs::read_to_string(&deploy);
    assert!(content.is_ok(), "failed to read {}", deploy.display());
    let content = content.unwrap_or_default();

    let apt_idx = content.find("Install setup apt packages");
    let users_idx = content.find("Ensure deploy user exists");
    let rustup_idx = content.find("Install rustup and cargo");

    assert!(apt_idx.is_some(), "shared setup deploy must install setup apt packages\n{content}");
    assert!(users_idx.is_some(), "shared setup deploy must include user setup\n{content}");
    assert!(rustup_idx.is_some(), "shared setup deploy must start rustup bootstrap\n{content}");
    assert!(apt_idx < rustup_idx, "setup apt packages should run before rustup bootstrap\n{content}");
    assert!(rustup_idx < users_idx, "rustup bootstrap should run before user setup\n{content}");
}

/// Leaves per-site runtime roles out of the shared setup deploy.
#[test]
fn shared_setup_deploy_keeps_runtime_roles_out() {
    let deploy = project_root().join("infra/setup.py");
    let content = fs::read_to_string(&deploy);
    assert!(content.is_ok(), "failed to read {}", deploy.display());
    let content = content.unwrap_or_default();

    assert!(
        !content.contains("bonesdeploy-nginx") && !content.contains("apparmor_parser"),
        "shared setup deploy must not apply per-site AppArmor\n{content}"
    );
    assert!(!content.contains("operations.py"), "shared setup deploy must not apply runtime roles\n{content}");
    assert!(!content.contains("per-site nginx"), "shared setup deploy must not apply per-site nginx\n{content}");
}

/// Applies runtime, `AppArmor`, and nginx through the dedicated runtime deploy script.
#[test]
fn remote_runtime_deploy_applies_runtime_apparmor_and_nginx() {
    let deploy = project_root().join("infra/runtime.py");
    let content = fs::read_to_string(&deploy);
    assert!(content.is_ok(), "failed to read {}", deploy.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("operations.py"),
        "runtime deploy must load template-specific operations module\n{content}"
    );
    assert!(
        content.contains("apparmor_parser") && content.contains("per-site nginx"),
        "runtime deploy must apply shared per-site AppArmor and nginx\n{content}"
    );
}

/// Uses a root-owned runtime socket directory so the PHP-FPM master can write without ACLs.
#[test]
fn remote_runtime_deploy_uses_root_owned_runtime_socket_dir() {
    let deploy = project_root().join("infra/runtime.py");
    let content = fs::read_to_string(&deploy);
    assert!(content.is_ok(), "failed to read {}", deploy.display());
    let content = content.unwrap_or_default();

    let socket_dir_start = content.find("name=\"Ensure socket directory exists\"").unwrap_or_else(|| {
        assert!(
            content.contains("name=\"Ensure socket directory exists\""),
            "runtime deploy must create the runtime socket directory"
        );
        0
    });
    let conf_dir_start = content.find("name=\"Ensure conf directory exists\"").unwrap_or_else(|| {
        assert!(
            content.contains("name=\"Ensure conf directory exists\""),
            "runtime deploy must configure the conf directory after the socket directory"
        );
        0
    });
    let socket_dir_block = &content[socket_dir_start..conf_dir_start];

    assert!(
        socket_dir_block.contains("path=PATHS[\"runtime_socket_dir\"]"),
        "runtime deploy must target the resolved runtime socket directory\n{socket_dir_block}"
    );
    assert!(
        socket_dir_block.contains("user=\"root\""),
        "runtime deploy must make the runtime socket directory root-owned\n{socket_dir_block}"
    );
    assert!(
        socket_dir_block.contains("group=DEPLOY_DATA[\"runtime_group\"]"),
        "runtime deploy must keep the runtime socket directory in the runtime group\n{socket_dir_block}"
    );
    assert!(
        socket_dir_block.contains("mode=\"0770\""),
        "runtime deploy must make the runtime socket directory group-writable without ACLs\n{socket_dir_block}"
    );
}

/// Installs runtime apt packages before applying runtime roles.
#[test]
fn remote_runtime_deploy_installs_packages_before_operations() {
    let deploy = project_root().join("infra/runtime.py");
    let content = fs::read_to_string(&deploy);
    assert!(content.is_ok(), "failed to read {}", deploy.display());
    let content = content.unwrap_or_default();

    let packages_idx = content.find("Install runtime apt packages");
    let ops_idx = content.find("operations.py");

    assert!(packages_idx.is_some(), "runtime deploy must install runtime apt packages\n{content}");
    assert!(ops_idx.is_some(), "runtime deploy must load template operations\n{content}");
    assert!(packages_idx < ops_idx, "runtime packages must install before template operations\n{content}");
}

/// Leaves SSL role out of the runtime deploy since SSL has its own deploy script.
#[test]
fn remote_runtime_deploy_excludes_ssl_logic() {
    let deploy = project_root().join("infra/runtime.py");
    let content = fs::read_to_string(&deploy);
    assert!(content.is_ok(), "failed to read {}", deploy.display());
    let content = content.unwrap_or_default();

    assert!(
        !content.contains("ssl_role") && !content.contains("certbot"),
        "runtime deploy must not include SSL logic - SSL has its own deploy script\n{content}"
    );
}

/// Applies SSL operations through the dedicated SSL deploy script.
#[test]
fn remote_ssl_deploy_applies_ssl_operations_only() {
    let deploy = project_root().join("infra/ssl.py");
    let content = fs::read_to_string(&deploy);
    assert!(content.is_ok(), "failed to read {}", deploy.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("certbot certonly") && content.contains("ssl_domain"),
        "SSL deploy must use certbot for certificate management\n{content}"
    );
    assert!(!content.contains("apparmor_parser"), "SSL deploy must not include AppArmor operations\n{content}");
    assert!(!content.contains("\"per-site nginx\""), "SSL deploy must not include general nginx role setup\n{content}");
    assert!(!content.contains("operations.py"), "SSL deploy must not include runtime role\n{content}");
}
