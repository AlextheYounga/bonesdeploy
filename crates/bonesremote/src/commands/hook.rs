use std::io::{Read, stdin};

use anyhow::{Context, Result};
use shared::config;
use shared::paths;

pub fn post_receive(site: &str) -> Result<()> {
    let mut stdin_buf = String::new();
    stdin().read_to_string(&mut stdin_buf).context("Failed to read post-receive stdin")?;

    let bones_path = paths::bonesremote_bones_toml_path(site);
    let cfg = config::load(&bones_path)
        .with_context(|| format!("Failed to load remote site state from {}", bones_path.display()))?;

    if !cfg.deploy_on_push {
        return Ok(());
    }

    let Some(newrev) = find_matching_revision(&stdin_buf, &cfg.branch) else {
        return Ok(());
    };

    super::deploy::run_full(site, Some(&newrev))
}

fn find_matching_revision(stdin_buf: &str, configured_branch: &str) -> Option<String> {
    for line in stdin_buf.lines() {
        let mut parts = line.splitn(3, char::is_whitespace);
        let _oldrev = parts.next()?;
        let newrev = parts.next()?;
        let ref_name = parts.next()?;

        let Some(branch_name) = ref_name.strip_prefix("refs/heads/") else {
            continue;
        };
        if branch_name != configured_branch {
            continue;
        }
        if newrev.chars().all(|ch| ch == '0') {
            println!("[bonesdeploy] Branch '{branch_name}' was deleted; skipping deploy.");
            return None;
        }
        return Some(newrev.to_string());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::find_matching_revision;

    #[test]
    fn resolves_newrev_for_configured_branch() {
        let stdin = "0000000000000000000000000000000000000000 abc123 refs/heads/master\n";
        assert_eq!(find_matching_revision(stdin, "master"), Some("abc123".into()));
    }

    #[test]
    fn ignores_non_matching_branch() {
        let stdin = "0000000000000000000000000000000000000000 abc123 refs/heads/develop\n";
        assert_eq!(find_matching_revision(stdin, "master"), None);
    }

    #[test]
    fn ignores_tags() {
        let stdin = "0000000000000000000000000000000000000000 abc123 refs/tags/v1.0\n";
        assert_eq!(find_matching_revision(stdin, "v1.0"), None);
    }
}
