use std::fs;

use anyhow::{Context, Result};

use crate::privileges;
use crate::release::SiteMutation;
use crate::release::state as release_state;

pub fn run(site: &str, keep: usize) -> Result<()> {
    privileges::ensure_root("bonesremote release prune")?;
    let mutation = SiteMutation::acquire(site)?;
    run_locked(&mutation, keep)
}

pub fn run_locked(mutation: &SiteMutation, keep: usize) -> Result<()> {
    let project_root = &mutation.config().project_root;

    let pruned = prune_old_releases(project_root, keep)?;
    if !pruned.is_empty() {
        println!("Pruned releases: {}", pruned.join(", "));
    }

    Ok(())
}

pub fn prune_old_releases(project_root: &str, keep: usize) -> Result<Vec<String>> {
    let active_release = release_state::current_release_name(project_root)?;
    let releases = release_state::list_releases_sorted(project_root)?;
    let keep = keep.max(1);

    // Compute the plan before touching the filesystem. The active release is
    // never a candidate, so an active release that happens to be the oldest
    // must not stall pruning (the old remove/push-back loop selected it forever).
    let excess = releases.len().saturating_sub(keep);
    let candidates = releases.into_iter().filter(|release| release != &active_release).take(excess);

    let mut pruned = Vec::new();
    for release in candidates {
        let path = release_state::release_dir(project_root, &release);
        if path.exists() {
            fs::remove_dir_all(&path).with_context(|| format!("Failed to prune old release {}", path.display()))?;
            pruned.push(release);
        }
    }

    Ok(pruned)
}
