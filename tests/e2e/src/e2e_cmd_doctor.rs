use anyhow::Result;

use crate::support::{cli, repo};

#[test]
#[ignore = "e2e test"]
fn e2e_bonesdeploy_doctor_local_passes_for_minimal_valid_layout() -> Result<()> {
    let sandbox = repo::create_temp_git_repo()?;
    repo::write_minimal_bones_project(&sandbox.path)?;
    let init = cli::run_bonesdeploy(&sandbox.path, ["init"])?;
    cli::assert_success(&init)?;

    let output = cli::run_bonesdeploy(&sandbox.path, ["doctor", "--local"])?;
    cli::assert_success(&output)?;
    cli::assert_stdout_contains(&output, "All checks passed")?;

    Ok(())
}
