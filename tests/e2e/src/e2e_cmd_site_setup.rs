use anyhow::Result;

use crate::support::{cli, fakes, repo};

#[test]
#[ignore = "e2e test"]
fn e2e_bonesdeploy_site_setup_invokes_ansible_flow() -> Result<()> {
    let sandbox = repo::create_temp_git_repo()?;
    repo::write_minimal_bones_project(&sandbox.path)?;
    let fake_bin = fakes::FakeCommandBin::with_ansible_playbook_and_ssh()?;

    let output = cli::run_bonesdeploy_with_env(&sandbox.path, ["site", "setup"], [("PATH", fake_bin.path())])?;
    cli::assert_success(&output)?;
    cli::assert_stdout_contains(&output, "Running site setup against 127.0.0.1 as root")?;
    cli::assert_stdout_contains(&output, "Site setup complete")?;

    let ssh_invocation = fake_bin.ssh_invocation()?;
    assert!(ssh_invocation.contains("-p 2222"));
    assert!(ssh_invocation.contains("root@127.0.0.1"));

    let ansible_invocation = fake_bin.ansible_invocation()?;
    assert!(ansible_invocation.contains("-i 127.0.0.1,"));
    assert!(ansible_invocation.contains("-u root"));
    assert!(ansible_invocation.contains(".bones/site/playbooks/setup.yml"));

    Ok(())
}
