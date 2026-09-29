use anyhow::Result;
use bonesdeploy_core::config::{
    Bones, RUNTIME_PYTHON_VERSION, RUNTIME_RUBY_VERSION, RemoteDeploymentConfig, RemoteRuntime, RuntimeBackend,
};
use bonesdeploy_core::paths;

#[test]
fn docker_remote_deployment_config_contains_only_consumed_values() -> Result<()> {
    let mut bones = Bones::for_site("mysite");
    bones.branch = "main".to_string();
    bones.releases_keep = 3;
    bones.runtime.backend = RuntimeBackend::Docker;
    bones.runtime.web_root = "public".to_string();
    bones.runtime.template = "rails".to_string();
    bones.runtime.node_version = "22.0.0".to_string();
    bones.runtime.permissions = Some(toml::Value::String("ignored".to_string()));
    bones.runtime.compose_port = Some(8080);
    bones.runtime.compose_wait_timeout = 240;
    bones.runtime.extra.insert(RUNTIME_PYTHON_VERSION.to_string(), toml::Value::String("3.14".to_string()));
    bones.runtime.extra.insert(RUNTIME_RUBY_VERSION.to_string(), toml::Value::String("3.4.8".to_string()));
    bones.runtime.extra.insert("workers".to_string(), toml::Value::Integer(3));
    bones.build.timeout_seconds = 120;
    bones.host = "example.com".to_string();
    bones.ssh_user = "root".to_string();
    bones.port = "2222".to_string();
    bones.domain = "myapp.com".to_string();
    bones.ssl_enabled = true;
    bones.backup.passphrase = "hex-passphrase".to_string();

    let descriptor = RemoteDeploymentConfig::from_bones(&bones);

    assert_eq!(
        serde_json::to_value(&descriptor)?,
        serde_json::json!({
            "releases_keep": 3,
            "runtime": {
                "backend": "docker",
                "compose_port": 8080,
                "compose_wait_timeout": 240
            }
        })
    );
    Ok(())
}

#[test]
fn native_remote_deployment_config_contains_only_consumed_values() -> Result<()> {
    let mut bones = Bones::for_site("atlas");
    bones.releases_keep = 7;
    bones.runtime.web_root = "dist".to_string();
    bones.runtime.template = "rails".to_string();
    bones.runtime.node_version = "22.0.0".to_string();
    bones.runtime.compose_port = Some(8080);
    bones.runtime.compose_wait_timeout = 240;
    bones.runtime.extra.insert(RUNTIME_PYTHON_VERSION.to_string(), toml::Value::String("3.14".to_string()));
    bones.runtime.extra.insert(RUNTIME_RUBY_VERSION.to_string(), toml::Value::String("3.4.8".to_string()));
    bones.runtime.extra.insert("workers".to_string(), toml::Value::Integer(3));

    let descriptor = RemoteDeploymentConfig::from_bones(&bones);

    assert_eq!(
        serde_json::to_value(&descriptor)?,
        serde_json::json!({
            "releases_keep": 7,
            "runtime": {
                "backend": "native",
                "web_root": "dist",
                "ruby_version": "3.4.8"
            }
        })
    );
    Ok(())
}

#[test]
fn remote_deployment_config_round_trips_through_json() -> Result<()> {
    let mut bones = Bones::for_site("atlas");
    bones.releases_keep = 7;
    bones.runtime.web_root = "dist".to_string();
    let descriptor = RemoteDeploymentConfig::from_bones(&bones);
    let json = serde_json::to_string(&descriptor)?;
    let restored: RemoteDeploymentConfig = serde_json::from_str(&json)?;

    assert_eq!(restored.releases_keep, 7);
    assert_eq!(restored.runtime, RemoteRuntime::Native { web_root: "dist".to_string(), ruby_version: None });
    Ok(())
}

#[test]
fn remote_deployment_config_into_site_config_derives_identity_from_site() {
    let mut bones = Bones::for_site("original");
    bones.releases_keep = 2;
    bones.runtime.web_root = "build".to_string();
    bones.runtime.extra.insert(RUNTIME_RUBY_VERSION.to_string(), toml::Value::String("3.4.8".to_string()));
    bones.runtime.extra.insert(RUNTIME_PYTHON_VERSION.to_string(), toml::Value::String("3.14".to_string()));

    let descriptor = RemoteDeploymentConfig::from_bones(&bones);
    let site_config = descriptor.into_site_config("target-site");

    assert_eq!(site_config.project_name, "target-site");
    assert_eq!(site_config.project_root, paths::default_project_root_for("target-site"));
    assert_eq!(site_config.releases_keep, 2);
    assert_eq!(site_config.runtime.web_root, "build");
    assert_eq!(site_config.runtime.extra.len(), 1);
    assert_eq!(site_config.runtime.extra.get(RUNTIME_RUBY_VERSION).and_then(toml::Value::as_str), Some("3.4.8"));
    assert!(site_config.host.is_empty());
    assert_eq!(site_config.ssh_user, "root");
}

#[test]
fn remote_deployment_config_rejects_unknown_fields() {
    let json = r#"{"releases_keep":5,"runtime":{"backend":"native","web_root":"public"},"extra_field":"bad"}"#;
    let result: Result<RemoteDeploymentConfig, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

#[test]
fn remote_deployment_config_rejects_old_broad_runtime() {
    let json = r#"{"releases_keep":5,"runtime":{"backend":"native","template":"rails","web_root":"public","node_version":"24.19.0","permissions":null,"python_version":"3.14"}}"#;
    let result: Result<RemoteDeploymentConfig, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

#[test]
fn remote_deployment_config_validates_docker_limits() {
    let invalid_port = RemoteDeploymentConfig {
        releases_keep: 5,
        runtime: RemoteRuntime::Docker { compose_port: Some(0), compose_wait_timeout: 120 },
    };
    let invalid_timeout = RemoteDeploymentConfig {
        releases_keep: 5,
        runtime: RemoteRuntime::Docker { compose_port: None, compose_wait_timeout: 3601 },
    };

    assert!(invalid_port.validate().is_err());
    assert!(invalid_timeout.validate().is_err());
}

#[test]
fn remote_deployment_config_rejects_web_roots_outside_the_release() {
    for web_root in ["/var/www/public", "../public", "public/../../shared"] {
        let descriptor = RemoteDeploymentConfig {
            releases_keep: 5,
            runtime: RemoteRuntime::Native { web_root: web_root.to_string(), ruby_version: None },
        };

        assert!(descriptor.validate().is_err(), "accepted unsafe web root {web_root}");
    }
}
