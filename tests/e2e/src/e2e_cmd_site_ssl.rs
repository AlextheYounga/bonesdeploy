use anyhow::Result;

use crate::support::{cli, repo};

#[test]
#[ignore = "e2e test"]
fn e2e_bonesdeploy_site_ssl_persists_ssl_values_before_running_ansible() -> Result<()> {
    let sandbox = repo::create_temp_git_repo()?;
    repo::write_minimal_bones_project(&sandbox.path)?;

    let output = cli::run_bonesdeploy(&sandbox.path, ["site", "ssl", "--domain", "app.test", "--email", "ops@test"])?;
    cli::assert_failure(&output)?;

    repo::assert_bones_yaml_contains(&sandbox.path, "domain: app.test")?;
    repo::assert_bones_yaml_contains(&sandbox.path, "email: ops@test")?;

    Ok(())
}
