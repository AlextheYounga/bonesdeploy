use anyhow::Result;
use serde_json::{Map, Value};
use shared::config as shared_config;
use shared::paths::{ssl_certificate_key_path, ssl_certificate_path};

use crate::config;

fn base(cfg: &config::BonesConfig, web_root: &str) -> Result<Map<String, Value>> {
    let paths = cfg.data.deployment_paths(web_root);
    let mut vars = Map::new();

    vars.insert(String::from("ssh_port"), Value::String(cfg.data.port.clone()));
    vars.insert(String::from("deploy_user"), Value::String(shared_config::default_deploy_user()));
    vars.insert(
        String::from("runtime_user"),
        Value::String(shared_config::runtime_user_for(&cfg.data.project_name)),
    );
    vars.insert(
        String::from("runtime_group"),
        Value::String(shared_config::runtime_group_for(&cfg.data.project_name)),
    );
    vars.insert(
        String::from("release_group"),
        Value::String(shared_config::release_group_for(&cfg.data.project_name)),
    );
    vars.insert(String::from("project_root_parent"), Value::String(paths.project_root_parent.clone()));
    vars.insert(String::from("project_root"), Value::String(cfg.data.project_root.clone()));
    vars.insert(String::from("web_root"), Value::String(web_root.to_string()));
    vars.insert(String::from("project_name"), Value::String(cfg.data.project_name.clone()));
    vars.insert(String::from("repo_path"), Value::String(cfg.data.repo_path.clone()));
    vars.insert(String::from("paths"), serde_json::to_value(paths)?);

    Ok(vars)
}

pub fn setup(cfg: &config::BonesConfig, web_root: &str, deploy_authorized_key: &str) -> Result<Value> {
    let mut vars = base(cfg, web_root)?;
    vars.insert(String::from("deploy_authorized_key"), Value::String(deploy_authorized_key.to_string()));
    vars.insert(String::from("setup_label"), Value::String(String::from("bonesdeploy")));
    Ok(Value::Object(vars))
}

pub fn ssl(cfg: &config::BonesConfig, web_root: &str, domain: &str, email: &str) -> Result<Value> {
    let mut vars = base(cfg, web_root)?;
    vars.insert(String::from("ssl_domain"), Value::String(domain.to_string()));
    vars.insert(String::from("ssl_email"), Value::String(email.to_string()));
    vars.insert(String::from("nginx_ssl_certificate_path"), Value::String(ssl_certificate_path(domain)));
    vars.insert(String::from("nginx_ssl_certificate_key_path"), Value::String(ssl_certificate_key_path(domain)));
    Ok(Value::Object(vars))
}

#[cfg(test)]
mod tests {
    use crate::config::{BonesConfig, Data, Releases, Ssl};

    use super::ssl;

    fn test_cfg() -> BonesConfig {
        BonesConfig {
            data: Data {
                project_name: String::from("test"),
                repo_path: String::from("/home/git/test.git"),
                project_root: String::from("/srv/test"),
                host: String::from("example.com"),
                port: String::from("22"),
                branch: String::from("master"),
                remote_name: String::from("production"),
                deploy_on_push: true,
                ..Default::default()
            },
            releases: Releases::default(),
            ssl: Ssl::default(),
        }
    }

    /// Passes the SSL domain and email into the deploy data sent to the infra CLI.
    #[test]
    fn ssl_data_includes_domain_and_email() -> anyhow::Result<()> {
        let cfg = test_cfg();
        let vars = ssl(&cfg, "public", "app.example.com", "ops@example.com")?;

        assert_eq!(vars.get("ssl_domain"), Some(&serde_json::Value::String(String::from("app.example.com"))));
        assert_eq!(vars.get("ssl_email"), Some(&serde_json::Value::String(String::from("ops@example.com"))));
        Ok(())
    }
}
