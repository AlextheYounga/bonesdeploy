use std::fs;
use std::path::Path;
use std::process::id;

use anyhow::{Context, Result};
use bonesdeploy_core::paths;

use crate::config;

mod local;
mod remote;

struct Patch {
    id: &'static str,
    introduced_in: Version,
}

struct LocalPatchContext<'a> {
    marker_dir: &'a Path,
    patch: local::Context<'a>,
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

fn patches() -> [Patch; 2] {
    [
        Patch { id: local::CONFIG_REPO_ID, introduced_in: Version::new(0, 7, 3) },
        Patch { id: local::ROOT_CONFIG_REPO_ID, introduced_in: Version::new(0, 7, 3) },
    ]
}

pub(super) fn run_local(cfg: &config::Bones, target_version: &str) -> Result<()> {
    let target = Version::parse(target_version)?;
    let marker_dir = paths::bones_data_root().join("patches").join(&cfg.project_name);
    let context = LocalPatchContext {
        marker_dir: &marker_dir,
        patch: local::Context { cfg, bones_dir: Path::new(paths::LOCAL_BONES_DIR) },
    };
    run_local_patches(&patches(), target, &context)
}

fn run_local_patches(patches: &[Patch], target: Version, context: &LocalPatchContext<'_>) -> Result<()> {
    for patch in patches.iter().filter(|patch| patch.introduced_in <= target) {
        let marker = context.marker_dir.join(patch.id);
        if marker.exists() {
            continue;
        }

        local::apply(patch.id, &context.patch)?;
        write_marker(context.marker_dir, &marker)?;
    }
    Ok(())
}

pub(super) async fn run_remote(session: &openssh::Session, cfg: &config::Bones, target_version: &str) -> Result<()> {
    let target = Version::parse(target_version)?;
    remote::apply(session, cfg, target, &patches()).await
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
    use std::fs;

    use anyhow::Result;
    use tempfile::TempDir;

    use super::{LocalPatchContext, Version, local, patches, run_local_patches};

    fn config() -> crate::config::Bones {
        let mut cfg = crate::config::Bones::default();
        cfg.project_name = String::from("atlas");
        cfg.host = String::from("example.test");
        cfg.port = String::from("22");
        cfg
    }

    #[test]
    fn patches_are_ordered_by_registry() {
        let patches = patches();
        assert!(patches.windows(2).all(|pair| pair[0].introduced_in <= pair[1].introduced_in));
    }

    #[test]
    fn target_version_controls_patch_selection() {
        let patch = &patches()[0];
        assert!(Version::parse("0.7.3").is_ok_and(|version| patch.introduced_in <= version));
        assert!(Version::parse("0.7.2").is_ok_and(|version| patch.introduced_in > version));
    }

    #[test]
    fn versions_ignore_prerelease_suffixes() {
        assert_eq!(Version::parse("0.7.3-rc1").ok(), Some(Version::new(0, 7, 3)));
    }

    #[test]
    fn local_patches_create_and_update_the_config_repository_without_bash() -> Result<()> {
        let temp = TempDir::new()?;
        let bones_dir = temp.path().join(".bones");
        fs::create_dir(&bones_dir)?;
        let markers = temp.path().join("markers");
        let cfg = config();
        let context =
            LocalPatchContext { marker_dir: &markers, patch: local::Context { cfg: &cfg, bones_dir: &bones_dir } };

        run_local_patches(&patches(), Version::new(0, 7, 3), &context)?;

        let origin = std::process::Command::new("git")
            .args(["-C"])
            .arg(&bones_dir)
            .args(["remote", "get-url", "origin"])
            .output()?;
        assert_eq!(String::from_utf8(origin.stdout)?.trim(), local::config_repo_url(&cfg));
        assert!(markers.join(local::CONFIG_REPO_ID).exists());
        assert!(markers.join(local::ROOT_CONFIG_REPO_ID).exists());
        Ok(())
    }
}
