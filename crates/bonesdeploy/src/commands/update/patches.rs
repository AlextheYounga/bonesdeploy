use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio, id};

use anyhow::{Context, Result, bail};
use shared::paths;

use crate::config;
use crate::infra::ssh;

const LOCAL_CONFIG_REPO_PATCH: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/patches/local/0001-config-repo.sh"));
const REMOTE_CONFIG_REPO_PATCH: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/patches/remote/0001-config-repo.sh"));
const LOCAL_ROOT_CONFIG_REPO_PATCH: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/patches/local/0002-root-config-repo.sh"));
const REMOTE_ROOT_CONFIG_REPO_PATCH: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/patches/remote/0002-root-config-repo.sh"));

struct Patch {
    id: &'static str,
    introduced_in: Version,
    script: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Version {
    major: u64,
    minor: u64,
    patch: u64,
}

impl Version {
    const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self { major, minor, patch }
    }

    fn parse(value: &str) -> Result<Self> {
        let mut parts = value.split('.');
        let major = parse_component(parts.next(), value)?;
        let minor = parse_component(parts.next(), value)?;
        let patch = parse_component(parts.next(), value)?;
        Ok(Self { major, minor, patch })
    }
}

fn parse_component(component: Option<&str>, version: &str) -> Result<u64> {
    let component = component.and_then(|value| value.split_once('-').map_or(Some(value), |(number, _)| Some(number)));
    component
        .filter(|value| !value.is_empty())
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| anyhow::anyhow!("invalid version '{version}'"))
}

fn local_patches() -> [Patch; 2] {
    [
        Patch { id: "0001-config-repo", introduced_in: Version::new(0, 7, 3), script: LOCAL_CONFIG_REPO_PATCH },
        Patch {
            id: "0002-root-config-repo",
            introduced_in: Version::new(0, 7, 4),
            script: LOCAL_ROOT_CONFIG_REPO_PATCH,
        },
    ]
}

fn remote_patches() -> [Patch; 2] {
    [
        Patch { id: "0001-config-repo", introduced_in: Version::new(0, 7, 3), script: REMOTE_CONFIG_REPO_PATCH },
        Patch {
            id: "0002-root-config-repo",
            introduced_in: Version::new(0, 7, 4),
            script: REMOTE_ROOT_CONFIG_REPO_PATCH,
        },
    ]
}

pub(super) fn run_local(cfg: &config::Bones, target_version: &str) -> Result<()> {
    let target = Version::parse(target_version)?;
    let marker_dir = paths::bones_data_root().join("patches").join(&cfg.project_name);
    run_local_patches(&local_patches(), target, &marker_dir, cfg)
}

fn run_local_patches(patches: &[Patch], target: Version, marker_dir: &Path, cfg: &config::Bones) -> Result<()> {
    for patch in patches.iter().filter(|patch| patch.introduced_in <= target) {
        let marker = marker_dir.join(patch.id);
        if marker.exists() {
            continue;
        }

        let status = Command::new("bash")
            .arg("-s")
            .env("BONESDEPLOY_SITE", &cfg.project_name)
            .env("BONESDEPLOY_HOST", &cfg.host)
            .env("BONESDEPLOY_PORT", &cfg.port)
            .env("BONESDEPLOY_BONES_REPO", paths::default_bones_repo_path_for(&cfg.project_name))
            .stdin(Stdio::piped())
            .spawn()
            .with_context(|| format!("Failed to start local patch {}", patch.id))
            .and_then(|mut child| {
                use std::io::Write as _;
                let mut stdin = child.stdin.take().context("local patch stdin was not piped")?;
                stdin.write_all(patch.script.as_bytes()).context("Failed to write local patch")?;
                drop(stdin);
                child.wait().context("Failed to wait for local patch")
            })?;
        if !status.success() {
            bail!("Local patch {} failed", patch.id);
        }
        write_marker(&marker_dir, &marker)?;
    }
    Ok(())
}

pub(super) async fn run_remote(session: &openssh::Session, cfg: &config::Bones, target_version: &str) -> Result<()> {
    let target = Version::parse(target_version)?;
    let marker_dir = PathBuf::from("/var/lib/bonesdeploy/patches").join(&cfg.project_name);
    for patch in remote_patches().iter().filter(|patch| patch.introduced_in <= target) {
        let marker = marker_dir.join(patch.id);
        let command = format!(
            "if [ -e {marker} ]; then exit 0; fi; mkdir -p {parent}; env BONESDEPLOY_SITE={site} BONESDEPLOY_BONES_REPO={repo} bash -s && touch {marker}",
            marker = ssh::shell_quote(&marker.display().to_string()),
            parent = ssh::shell_quote(&marker_dir.display().to_string()),
            site = ssh::shell_quote(&cfg.project_name),
            repo = ssh::shell_quote(&paths::default_bones_repo_path_for(&cfg.project_name)),
        );
        ssh::run_cmd_with_stdin(session, &command, patch.script.as_bytes())
            .await
            .with_context(|| format!("Remote patch {} failed", patch.id))?;
    }
    Ok(())
}

fn write_marker(marker_dir: &Path, marker: &Path) -> Result<()> {
    fs::create_dir_all(marker_dir)
        .with_context(|| format!("Failed to create patch marker directory {}", marker_dir.display()))?;
    let temporary = marker.with_extension(format!("tmp-{}", id()));
    fs::write(&temporary, b"completed\n")
        .with_context(|| format!("Failed to write patch marker {}", temporary.display()))?;
    fs::rename(&temporary, marker).with_context(|| format!("Failed to activate patch marker {}", marker.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Version, local_patches};

    #[test]
    fn patches_are_ordered_by_registry() {
        let patches = local_patches();
        assert!(patches.windows(2).all(|pair| pair[0].introduced_in <= pair[1].introduced_in));
    }

    #[test]
    fn target_version_controls_patch_selection() {
        let patch = &local_patches()[0];
        assert!(Version::parse("0.7.3").is_ok_and(|version| patch.introduced_in <= version));
        assert!(Version::parse("0.7.2").is_ok_and(|version| patch.introduced_in > version));
        let migration = &local_patches()[1];
        assert!(Version::parse("0.7.4").is_ok_and(|version| migration.introduced_in <= version));
        assert!(Version::parse("0.7.3").is_ok_and(|version| migration.introduced_in > version));
    }

    #[test]
    fn versions_ignore_prerelease_suffixes() {
        assert_eq!(Version::parse("0.7.3-rc1").ok(), Some(Version::new(0, 7, 3)));
    }
}
