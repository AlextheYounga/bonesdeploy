use std::env;

pub(crate) fn resolve(config_ssh_user: Option<&str>) -> String {
    // env override takes highest precedence
    if let Ok(env_user) = env::var("BONES_BOOTSTRAP_SSH_USER") {
        let trimmed = env_user.trim().to_string();
        if !trimmed.is_empty() {
            return trimmed;
        }
    }
    resolve_from(config_ssh_user)
}

fn resolve_from(value: Option<&str>) -> String {
    value
        .and_then(|raw| {
            let trimmed = raw.trim().to_string();
            if trimmed.is_empty() { None } else { Some(trimmed) }
        })
        .unwrap_or_else(|| String::from("root"))
}

#[cfg(test)]
mod tests {
    use super::resolve_from;

    #[test]
    fn defaults_to_root() {
        let user = resolve_from(None);
        assert_eq!(user, "root");
    }

    #[test]
    fn uses_config_value() {
        let user = resolve_from(Some("ubuntu"));
        assert_eq!(user, "ubuntu");
    }

    #[test]
    fn trims_and_rejects_blank_values() {
        let user = resolve_from(Some("   "));
        assert_eq!(user, "root");

        let user = resolve_from(Some("  ubuntu  "));
        assert_eq!(user, "ubuntu");
    }
}
