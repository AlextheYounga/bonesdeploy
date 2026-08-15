use std::fs::{self, DirEntry};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use bonesdeploy_core::paths;

const OLD_LAYOUT_ENTRIES: [&str; 3] = ["deployment", "infra", "secrets"];
const OLD_CONTROL_ENTRIES: [&str; 3] = [".git", paths::GITIGNORE_FILE, "bones.toml"];

/// Move project-owned files out of the old `.bones` workspace.
pub fn run() -> Result<()> {
    migrate_at(Path::new("."))
}

fn migrate_at(project_root: &Path) -> Result<()> {
    let old_path = project_root.join(paths::OLD_BONES_DIR);
    let old_metadata = fs::symlink_metadata(&old_path)
        .with_context(|| format!("Cannot inspect old layout at {}", old_path.display()))?;
    let old_type = old_metadata.file_type();
    if old_type.is_symlink() {
        let source = fs::canonicalize(&old_path)
            .with_context(|| format!("Cannot resolve old layout at {}", old_path.display()))?;
        migrate_from(project_root, &old_path, &source, true)
    } else if old_type.is_dir() {
        migrate_from(project_root, &old_path, &old_path, false)
    } else {
        bail!(".bones is neither a directory nor a symlink; migration refused")
    }
}

fn migrate_from(project_root: &Path, old_path: &Path, source: &Path, old_is_link: bool) -> Result<()> {
    let source_type = fs::symlink_metadata(source)?.file_type();
    if !source_type.is_dir() {
        bail!("Old .bones target is not a directory; migration refused")
    }

    let destination = project_root.join("infra");
    if fs::symlink_metadata(&destination).is_ok() {
        bail!("{} already exists; migration will not merge or overwrite it", destination.display())
    }

    validate_source(source)?;

    let staging = project_root.join(format!(".infra.migrate-{}", migration_nonce()));
    if fs::symlink_metadata(&staging).is_ok() {
        bail!("Migration staging path already exists: {}", staging.display())
    }
    fs::create_dir(&staging).with_context(|| format!("Cannot create {}", staging.display()))?;

    if let Err(error) = copy_owned_content(source, &staging) {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }

    verify_owned_content(source, &staging)?;
    fs::rename(&staging, &destination)
        .with_context(|| format!("Cannot install migrated infrastructure at {}", destination.display()))?;
    verify_owned_content(source, &destination)?;

    if old_is_link {
        fs::remove_file(old_path).with_context(|| format!("Cannot remove {}", old_path.display()))?;
    } else {
        fs::remove_dir_all(old_path).with_context(|| format!("Cannot remove {}", old_path.display()))?;
    }

    println!("Migrated old .bones content into {}.", destination.display());
    println!("No Git commit was created; review and commit the changes when ready.");
    Ok(())
}

fn validate_source(source: &Path) -> Result<()> {
    for entry in fs::read_dir(source).with_context(|| format!("Cannot read {}", source.display()))? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !OLD_LAYOUT_ENTRIES.contains(&name.as_ref()) && !OLD_CONTROL_ENTRIES.contains(&name.as_ref()) {
            bail!("Unexpected entry {} in old .bones; migration is ambiguous", entry.path().display())
        }
        if !OLD_CONTROL_ENTRIES.contains(&name.as_ref()) {
            validate_tree(&entry)?;
        } else if entry.file_type()?.is_symlink() {
            bail!("Old control entry {} is a symlink; migration refused", entry.path().display())
        }
    }
    let old_infra = source.join("infra");
    if old_infra.is_dir() {
        for name in ["deployment", "secrets"] {
            if fs::symlink_metadata(old_infra.join(name)).is_ok() {
                bail!("Both .bones/infra/{name} and .bones/{name} exist; migration is ambiguous")
            }
        }
    }
    Ok(())
}

fn validate_tree(entry: &DirEntry) -> Result<()> {
    let file_type = entry.file_type()?;
    if file_type.is_symlink() || (!file_type.is_dir() && !file_type.is_file()) {
        bail!("Unsafe entry {} in old .bones; migration refused", entry.path().display())
    }
    if file_type.is_dir() {
        for child in fs::read_dir(entry.path())? {
            validate_tree(&child?)?;
        }
    }
    Ok(())
}

fn copy_owned_content(source: &Path, destination: &Path) -> Result<()> {
    if let Some(old_infra) = existing_directory(&source.join("infra"))? {
        copy_directory_contents(&old_infra, destination)?;
    }
    for name in ["deployment", "secrets"] {
        let source_entry = source.join(name);
        if fs::symlink_metadata(&source_entry).is_ok() {
            copy_tree(&source_entry, &destination.join(name))?;
        }
    }
    Ok(())
}

fn existing_directory(path: &Path) -> Result<Option<PathBuf>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_dir() => Ok(Some(path.to_path_buf())),
        Ok(_) => bail!("{} is not a directory; migration refused", path.display()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn copy_directory_contents(source: &Path, destination: &Path) -> Result<()> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        copy_tree(&entry.path(), &destination.join(entry.file_name()))?;
    }
    Ok(())
}

fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(source)?;
    let file_type = metadata.file_type();
    if file_type.is_dir() {
        fs::create_dir(destination).with_context(|| format!("Cannot create {}", destination.display()))?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            copy_tree(&entry.path(), &destination.join(entry.file_name()))?;
        }
        fs::set_permissions(destination, metadata.permissions())?;
    } else if file_type.is_file() {
        fs::copy(source, destination).with_context(|| format!("Cannot copy {}", source.display()))?;
        fs::set_permissions(destination, metadata.permissions())?;
    } else {
        bail!("Unsafe entry {} in old .bones; migration refused", source.display())
    }
    Ok(())
}

fn verify_owned_content(source: &Path, destination: &Path) -> Result<()> {
    if let Some(old_infra) = existing_directory(&source.join("infra"))? {
        verify_directory_contents(&old_infra, destination)?;
    }
    for name in ["deployment", "secrets"] {
        let source_entry = source.join(name);
        if fs::symlink_metadata(&source_entry).is_ok() {
            verify_tree(&source_entry, &destination.join(name))?;
        }
    }
    Ok(())
}

fn verify_directory_contents(source: &Path, destination: &Path) -> Result<()> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        verify_tree(&entry.path(), &destination.join(entry.file_name()))?;
    }
    Ok(())
}

fn verify_tree(source: &Path, destination: &Path) -> Result<()> {
    let source_metadata = fs::symlink_metadata(source)?;
    let destination_metadata = fs::symlink_metadata(destination)
        .with_context(|| format!("Migration destination is incomplete: {}", destination.display()))?;
    let source_type = source_metadata.file_type();
    let destination_type = destination_metadata.file_type();
    if source_type.is_dir() != destination_type.is_dir() || source_type.is_file() != destination_type.is_file() {
        bail!("Migration destination has an unsafe type for {}", destination.display())
    }
    if source_type.is_file() {
        let source_bytes = fs::read(source)?;
        let destination_bytes = fs::read(destination)?;
        if source_bytes != destination_bytes {
            bail!("Migration verification failed for {}", destination.display())
        }
    } else {
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            verify_tree(&entry.path(), &destination.join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn migration_nonce() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::symlink;
    use tempfile::TempDir;

    use anyhow::Result;

    use super::migrate_at;

    fn project() -> Result<TempDir> {
        Ok(tempfile::tempdir()?)
    }

    #[test]
    fn migrates_owned_content_and_preserves_ciphertext_without_commit() -> Result<()> {
        let project = project()?;
        let old = project.path().join(".bones");
        fs::create_dir_all(old.join("infra/templates"))?;
        fs::create_dir_all(old.join("deployment/build"))?;
        fs::create_dir_all(old.join("secrets"))?;
        fs::write(old.join("infra/templates/site.conf"), "template")?;
        fs::write(old.join("deployment/build/01_build.sh"), "build")?;
        let ciphertext = vec![0, 159, 42, 255];
        fs::write(old.join("secrets/.env.gpg"), &ciphertext)?;
        fs::write(old.join("bones.toml"), "must not be copied")?;

        migrate_at(project.path())?;

        assert_eq!(fs::read(project.path().join("infra/secrets/.env.gpg"))?, ciphertext);
        assert!(project.path().join("infra/templates/site.conf").is_file());
        assert!(project.path().join("infra/deployment/build/01_build.sh").is_file());
        assert!(!project.path().join(".bones").exists());
        assert!(!project.path().join("infra/bones.toml").exists());
        Ok(())
    }

    #[test]
    fn migrates_symlink_without_removing_its_target() -> Result<()> {
        let project = project()?;
        let target = project.path().join("config-root");
        fs::create_dir_all(target.join("deployment"))?;
        fs::write(target.join("deployment/script.sh"), "run")?;
        symlink(&target, project.path().join(".bones"))?;

        migrate_at(project.path())?;

        assert!(!project.path().join(".bones").exists());
        assert!(target.is_dir());
        assert!(project.path().join("infra/deployment/script.sh").is_file());
        Ok(())
    }

    #[test]
    fn refuses_existing_destination_and_ambiguous_entries_before_changes() -> Result<()> {
        let project = project()?;
        let old = project.path().join(".bones");
        fs::create_dir_all(&old)?;
        fs::write(old.join("unexpected.txt"), "unsafe")?;
        assert!(migrate_at(project.path()).is_err());
        assert!(old.exists());

        fs::remove_file(old.join("unexpected.txt"))?;
        fs::create_dir(project.path().join("infra"))?;
        assert!(migrate_at(project.path()).is_err());
        assert!(old.exists());
        Ok(())
    }
}
