use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use bonesdeploy::build::inventory::EntryType;
use bonesdeploy::build::native::build_scripts;
use bonesdeploy::infra::git;

#[test]
fn exporting_a_configured_branch_uses_only_its_committed_tree() -> Result<()> {
    let repo = tempfile::tempdir()?;
    run(repo.path(), ["init", "--initial-branch=main"])?;
    run(repo.path(), ["config", "user.email", "test@example.com"])?;
    run(repo.path(), ["config", "user.name", "Test"])?;
    fs::write(repo.path().join("tracked.txt"), "committed")?;
    fs::write(repo.path().join(".gitignore"), "ignored.txt\n")?;
    let build_dir = repo.path().join("infra/deployment/build");
    fs::create_dir_all(&build_dir)?;
    fs::write(build_dir.join("01_build.sh"), "echo committed")?;
    #[cfg(unix)]
    fs::set_permissions(build_dir.join("01_build.sh"), fs::Permissions::from_mode(0o755))?;
    fs::write(repo.path().join("infra/deployment/bundle.txt"), "committed bundle")?;
    run(repo.path(), ["add", "."])?;
    run(repo.path(), ["commit", "-m", "initial"])?;
    fs::write(repo.path().join("tracked.txt"), "dirty")?;
    fs::write(repo.path().join("untracked.txt"), "untracked")?;
    fs::write(repo.path().join("ignored.txt"), "ignored")?;
    fs::write(build_dir.join("01_build.sh"), "echo dirty")?;
    fs::write(repo.path().join("infra/deployment/bundle.txt"), "dirty bundle")?;

    let revision = git::resolve_branch_commit(repo.path(), "main")?;
    let export = tempfile::tempdir()?;
    let inventory = git::export_commit(repo.path(), &revision, export.path())?;

    assert_eq!(fs::read_to_string(export.path().join("tracked.txt"))?, "committed");
    assert!(!export.path().join("untracked.txt").exists());
    assert!(!export.path().join("ignored.txt").exists());
    assert!(!export.path().join(".git").exists());
    assert_eq!(inventory.get(Path::new("tracked.txt")).map(|entry| entry.entry_type), Some(EntryType::File));
    assert_eq!(
        inventory.get(Path::new("infra/deployment/build")).map(|entry| entry.entry_type),
        Some(EntryType::Directory)
    );
    #[cfg(unix)]
    assert!(
        inventory.get(Path::new("infra/deployment/build/01_build.sh")).is_some_and(|entry| entry.mode & 0o111 != 0)
    );
    let (deployment, scripts) = build_scripts(export.path())?.context("committed build script expected")?;
    assert_eq!(scripts, vec![export.path().join("infra/deployment/build/01_build.sh")]);
    assert_eq!(fs::read_to_string(&scripts[0])?, "echo committed");
    assert_eq!(fs::read_to_string(deployment.join("bundle.txt"))?, "committed bundle");
    Ok(())
}

fn run<const N: usize>(repo: &Path, arguments: [&str; N]) -> Result<()> {
    let status = Command::new("git").arg("-C").arg(repo).args(arguments).status().context("run git")?;
    anyhow::ensure!(status.success(), "git command failed");
    Ok(())
}
