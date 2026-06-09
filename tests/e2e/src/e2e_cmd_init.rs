use anyhow::Result;

use crate::support::{cli, repo};

/// Verifies that `bonesdeploy init` reuses an existing scaffold and creates the pre-push symlink.
#[test]
#[ignore = "e2e test"]
fn e2e_bonesdeploy_init_reuses_existing_scaffold_and_symlinks_pre_push_hook() -> Result<()> {
    let sandbox = repo::create_temp_git_repo()?;
    repo::write_minimal_bones_project(&sandbox.path)?;

    let output = cli::run_bonesdeploy_with_input(&sandbox.path, ["init"], "n\n")?;
    cli::assert_success(&output)?;
    repo::assert_pre_push_symlink_exists(&sandbox.path)?;

    Ok(())
}
