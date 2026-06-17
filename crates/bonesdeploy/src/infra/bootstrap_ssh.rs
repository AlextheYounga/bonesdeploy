use std::env;

pub(crate) fn resolve() -> String {
    resolve_from(env::var("BONES_BOOTSTRAP_SSH_USER").ok())
}

fn resolve_from(value: Option<String>) -> String {
    value.map(|raw| raw.trim().to_string()).filter(|raw| !raw.is_empty()).unwrap_or_else(|| String::from("root"))
}

#[cfg(test)]
mod tests {
    use super::resolve_from;

    /// Defaults the bootstrap SSH user to root when no override is provided.
    #[test]
    fn defaults_to_root() {
        let user = resolve_from(None);
        assert_eq!(user, "root");
    }

    /// Uses the environment override when provided for the bootstrap SSH user.
    #[test]
    fn uses_env_override() {
        let user = resolve_from(Some(String::from("ubuntu")));
        assert_eq!(user, "ubuntu");
    }

    /// Trims whitespace and falls back to root when the bootstrap SSH user is blank.
    #[test]
    fn trims_and_rejects_blank_values() {
        let user = resolve_from(Some(String::from("   ")));
        assert_eq!(user, "root");

        let user = resolve_from(Some(String::from("  ubuntu  ")));
        assert_eq!(user, "ubuntu");
    }
}
