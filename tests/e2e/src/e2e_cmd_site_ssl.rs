use anyhow::Result;

use crate::support::{cli, fakes, repo};

#[test]
#[ignore = "e2e test"]
fn e2e_bonesdeploy_site_ssl_persists_ssl_values_before_running_ansible() -> Result<()> {
    let sandbox = repo::create_temp_git_repo()?;
    repo::write_minimal_bones_project(&sandbox.path)?;
    let fake_bin = fakes::FakeCommandBin::with_ansible_playbook_and_ssh()?;

    let output = cli::run_bonesdeploy_with_env(
        &sandbox.path,
        ["site", "ssl", "--domain", "app.test", "--email", "ops@test"],
        [("PATH", fake_bin.path())],
    )?;
    cli::assert_success(&output)?;
    cli::assert_stdout_contains(&output, "Running site ssl against 127.0.0.1 for app.test")?;
    cli::assert_stdout_contains(&output, "SSL setup complete")?;

    repo::assert_bones_yaml_contains(&sandbox.path, "domain: app.test")?;
    repo::assert_bones_yaml_contains(&sandbox.path, "email: ops@test")?;
    repo::assert_bones_yaml_contains(&sandbox.path, "enabled: true")?;

    let ssh_invocation = fake_bin.ssh_invocation()?;
    assert!(ssh_invocation.contains("root@127.0.0.1"));

    let ansible_invocation = fake_bin.ansible_invocation()?;
    assert!(ansible_invocation.contains("--tags nginx,ssl"));
    assert!(ansible_invocation.contains("ssl_domain=app.test"));
    assert!(ansible_invocation.contains("ssl_email=ops@test"));

    Ok(())
}
