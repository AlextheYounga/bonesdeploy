use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;

use bonesremote::release::lifecycle::stage::{create_release_name, create_unique_release_dir_with};

fn temp_dir(prefix: &str) -> Result<PathBuf> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
    let path = env::temp_dir().join(format!("{prefix}_{}_{}", process::id(), nanos));
    fs::create_dir_all(&path)?;
    Ok(path)
}

#[test]
fn release_name_embeds_commit_prefix_and_suffix() -> Result<()> {
    let name = create_release_name("46a0b75cdeadbeef0123456789abcdef01234567", "a7f2")?;

    assert!(name.ends_with("-46a0b75c-a7f2"), "unexpected name: {name}");
    assert_eq!(name.len(), "20260804_190321".len() + "-46a0b75c-a7f2".len());
    assert_eq!(name.as_bytes()[8], b'_');
    assert_eq!(name.matches('-').count(), 2);
    Ok(())
}

#[test]
fn release_name_tolerates_short_revision_commits() -> Result<()> {
    let name = create_release_name("abc", "1234")?;
    assert!(name.ends_with("-abc-1234"), "unexpected name: {name}");
    Ok(())
}

#[test]
fn unique_release_dir_retries_on_collision() -> Result<()> {
    let root = temp_dir("bonesremote_stage_collision")?;
    let releases = root.join("releases");
    fs::create_dir_all(&releases)?;
    // A directory already exists for the first candidate name, so the
    // first attempt must collide and the exclusive create must retry.
    fs::create_dir(releases.join("20260804_190321-46a0b75c-0000"))?;
    let project_root = root.to_string_lossy().into_owned();

    let mut candidates =
        ["20260804_190321-46a0b75c-0000".to_string(), "20260804_190321-46a0b75c-1000".to_string()].into_iter();

    let name = create_unique_release_dir_with(&project_root, &mut || {
        candidates.next().map_or_else(|| anyhow::bail!("exhausted test names"), Ok)
    })?;

    assert_eq!(name, "20260804_190321-46a0b75c-1000");
    assert!(releases.join("20260804_190321-46a0b75c-0000").exists());
    assert!(releases.join("20260804_190321-46a0b75c-1000").exists());

    fs::remove_dir_all(root).ok();
    Ok(())
}
