use std::collections::HashMap;
use std::ffi::{CString, OsString};
use std::fs::{self, OpenOptions};
use std::io::{self, Error, Read, Seek};
use std::mem::MaybeUninit;
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, bail};
use bonesdeploy_core::paths;
use zip::ZipArchive;
use zip::read::ZipFile;

const FILE_TYPE_MASK: u32 = 0o170_000;
const REGULAR_FILE: u32 = 0o100_000;
const DIRECTORY: u32 = 0o040_000;
const SYMBOLIC_LINK: u32 = 0o120_000;

#[derive(Clone, Copy, Debug)]
pub struct ArchiveLimits {
    pub max_entries: usize,
    pub max_path_bytes: usize,
    pub max_file_bytes: u64,
    pub max_expanded_bytes: u64,
    pub min_free_bytes: u64,
}

impl Default for ArchiveLimits {
    fn default() -> Self {
        Self {
            max_entries: 200_000,
            max_path_bytes: 4 * 1024,
            max_file_bytes: 16 * 1024 * 1024 * 1024,
            max_expanded_bytes: 64 * 1024 * 1024 * 1024,
            min_free_bytes: 512 * 1024 * 1024,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EntryKind {
    Directory,
    File,
    Symlink,
}

struct Entry {
    index: usize,
    path: PathBuf,
    kind: EntryKind,
    executable: bool,
    size: u64,
    link_target: Option<PathBuf>,
}

pub fn extract_archive<R: Read + Seek>(reader: R, destination: &Path, limits: ArchiveLimits) -> Result<()> {
    let metadata = fs::symlink_metadata(destination)
        .with_context(|| format!("Failed to inspect shared import destination {}", destination.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("Shared import destination is not a real directory: {}", destination.display());
    }

    let mut archive = ZipArchive::new(reader).context("Shared import is not a valid ZIP archive")?;
    let (entries, expanded_bytes) = validate_entries(&mut archive, limits)?;
    ensure_free_space(destination, expanded_bytes, limits.min_free_bytes)?;
    materialize_entries(&mut archive, destination, &entries)?;
    Ok(())
}

fn validate_entries<R: Read + Seek>(archive: &mut ZipArchive<R>, limits: ArchiveLimits) -> Result<(Vec<Entry>, u64)> {
    if archive.is_empty() {
        bail!("Shared import archive is empty");
    }
    if archive.len() > limits.max_entries {
        bail!("Shared import archive has more than {} entries", limits.max_entries);
    }

    let mut registered = HashMap::new();
    let mut entries = Vec::with_capacity(archive.len());
    let mut expanded = 0_u64;
    let mut saw_shared_root = false;

    for index in 0..archive.len() {
        let mut file = archive.by_index(index).context("Failed to read ZIP entry")?;
        if file.encrypted() {
            bail!("Shared import archive contains an encrypted entry");
        }

        let full_path = checked_path(file.name(), file.name_raw().len(), limits.max_path_bytes)?;
        let path = full_path
            .strip_prefix(paths::SHARED_DIR)
            .with_context(|| format!("Archive entry is not rooted beneath shared/: {}", full_path.display()))?
            .to_path_buf();
        let kind = entry_kind(&file)?;
        if path.as_os_str().is_empty() {
            if kind != EntryKind::Directory {
                bail!("The top-level shared archive entry must be a directory");
            }
            saw_shared_root = true;
        }
        register_path(&mut registered, &full_path, kind)?;

        let size = file.size();
        if kind != EntryKind::Directory {
            if size > limits.max_file_bytes {
                bail!("Archive entry {} exceeds {} bytes", full_path.display(), limits.max_file_bytes);
            }
            expanded = expanded.checked_add(size).context("Shared import expanded size overflow")?;
            if expanded > limits.max_expanded_bytes {
                bail!("Shared import archive exceeds {} expanded bytes", limits.max_expanded_bytes);
            }
        }

        if path == Path::new(paths::DOT_ENV) && kind != EntryKind::File {
            bail!("Archive shared/.env must be a regular file");
        }

        let link_target = if kind == EntryKind::Symlink {
            let target_size = usize::try_from(size).context("Archive symlink target is too large")?;
            if target_size > limits.max_path_bytes {
                bail!("Archive symlink target exceeds {} bytes", limits.max_path_bytes);
            }
            let mut bytes = Vec::with_capacity(target_size);
            (&mut file).take(size + 1).read_to_end(&mut bytes).context("Failed to read archive symlink target")?;
            if bytes.len() as u64 != size {
                bail!("Archive symlink size does not match its ZIP metadata");
            }
            let target = PathBuf::from(OsString::from_vec(bytes));
            checked_link_target(&path, &target, limits.max_path_bytes)?;
            Some(target)
        } else {
            None
        };

        entries.push(Entry {
            index,
            path,
            kind,
            executable: file.unix_mode().is_some_and(|mode| mode & 0o110 != 0),
            size,
            link_target,
        });
    }

    if !saw_shared_root && !entries.iter().any(|entry| !entry.path.as_os_str().is_empty()) {
        bail!("Shared import archive has no shared/ contents");
    }
    Ok((entries, expanded))
}

fn materialize_entries<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    destination: &Path,
    entries: &[Entry],
) -> Result<()> {
    let mut directories = Vec::new();
    for entry in entries.iter().filter(|entry| entry.kind != EntryKind::Symlink) {
        if entry.path.as_os_str().is_empty() || entry.path == Path::new(paths::DOT_ENV) {
            continue;
        }
        let target = destination.join(&entry.path);
        match entry.kind {
            EntryKind::Directory => {
                fs::create_dir_all(&target)
                    .with_context(|| format!("Failed to create imported directory {}", target.display()))?;
                directories.push(target);
            }
            EntryKind::File => {
                create_parent(&target)?;
                let mut output = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&target)
                    .with_context(|| format!("Failed to create imported file {}", target.display()))?;
                let mut input = archive.by_index(entry.index).context("Failed to reopen ZIP entry")?;
                let copied = io::copy(&mut input.by_ref().take(entry.size + 1), &mut output)
                    .with_context(|| format!("Failed to extract imported file {}", target.display()))?;
                if copied != entry.size {
                    bail!("Archive file size does not match ZIP metadata for {}", entry.path.display());
                }
                output.sync_all().with_context(|| format!("Failed to sync imported file {}", target.display()))?;
                let mode = if entry.executable { 0o750 } else { 0o640 };
                fs::set_permissions(&target, fs::Permissions::from_mode(mode))?;
            }
            EntryKind::Symlink => {}
        }
    }

    for directory in directories.into_iter().rev() {
        fs::set_permissions(directory, fs::Permissions::from_mode(0o750))?;
    }
    for entry in entries.iter().filter(|entry| entry.kind == EntryKind::Symlink) {
        let target = destination.join(&entry.path);
        create_parent(&target)?;
        let link_target = entry.link_target.as_ref().context("Validated symlink has no target")?;
        symlink(link_target, &target)
            .with_context(|| format!("Failed to create imported symlink {}", target.display()))?;
    }
    normalize_directory_modes(destination)?;
    Ok(())
}

fn checked_path(name: &str, raw_length: usize, max_path_bytes: usize) -> Result<PathBuf> {
    if raw_length > max_path_bytes {
        bail!("Archive path exceeds {max_path_bytes} bytes");
    }
    let path = Path::new(name);
    let mut checked = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => checked.push(value),
            _ => bail!("Archive path is not a direct relative child path: {name}"),
        }
    }
    if checked.as_os_str().is_empty() {
        bail!("Archive path is empty");
    }
    if checked.components().next().is_none_or(|component| component.as_os_str() != paths::SHARED_DIR) {
        bail!("Archive entry is not rooted beneath shared/: {name}");
    }
    Ok(checked)
}

fn entry_kind<R: Read>(file: &ZipFile<'_, R>) -> Result<EntryKind> {
    if let Some(mode) = file.unix_mode() {
        match mode & FILE_TYPE_MASK {
            DIRECTORY => return Ok(EntryKind::Directory),
            REGULAR_FILE => return Ok(EntryKind::File),
            SYMBOLIC_LINK => return Ok(EntryKind::Symlink),
            0 => {}
            _ => bail!("Shared import archive contains an unsupported special file"),
        }
    }
    if file.is_dir() { Ok(EntryKind::Directory) } else { Ok(EntryKind::File) }
}

fn register_path(entries: &mut HashMap<PathBuf, EntryKind>, path: &Path, kind: EntryKind) -> Result<()> {
    if entries.contains_key(path) {
        bail!("Shared import archive contains duplicate path {}", path.display());
    }
    for ancestor in path.ancestors().skip(1) {
        if let Some(ancestor_kind) = entries.get(ancestor)
            && *ancestor_kind != EntryKind::Directory
        {
            bail!("Archive path conflicts with non-directory {}", ancestor.display());
        }
    }
    if kind != EntryKind::Directory && entries.keys().any(|other| other != path && other.starts_with(path)) {
        bail!("Archive path conflicts with descendant {}", path.display());
    }
    entries.insert(path.to_path_buf(), kind);
    Ok(())
}

fn checked_link_target(path: &Path, target: &Path, max_path_bytes: usize) -> Result<()> {
    if target.as_os_str().len() > max_path_bytes {
        bail!("Archive symlink target exceeds {max_path_bytes} bytes");
    }
    if target.as_os_str().is_empty() {
        bail!("Archive symlink target is empty");
    }
    let mut depth = path.parent().map_or(0, |parent| parent.components().count());
    for component in target.components() {
        match component {
            Component::Normal(_) => depth += 1,
            Component::ParentDir if depth > 0 => depth -= 1,
            Component::ParentDir => bail!("Archive symlink escapes shared/"),
            Component::CurDir => {}
            _ => bail!("Archive symlink target is unsafe"),
        }
    }
    Ok(())
}

fn create_parent(path: &Path) -> Result<()> {
    let parent = path.parent().context("Imported archive path has no parent")?;
    fs::create_dir_all(parent).with_context(|| format!("Failed to create imported parent {}", parent.display()))
}

fn normalize_directory_modes(path: &Path) -> Result<()> {
    for entry in fs::read_dir(path).with_context(|| format!("Failed to read imported directory {}", path.display()))? {
        let entry = entry?;
        let metadata = fs::symlink_metadata(entry.path())?;
        if metadata.is_dir() {
            normalize_directory_modes(&entry.path())?;
        }
    }
    fs::set_permissions(path, fs::Permissions::from_mode(0o750))
        .with_context(|| format!("Failed to protect imported directory {}", path.display()))
}

fn ensure_free_space(path: &Path, required: u64, reserve: u64) -> Result<()> {
    let available = available_bytes(path)?;
    let required = required.checked_add(reserve).context("Shared import space requirement overflow")?;
    if available < required {
        bail!(
            "Shared import needs {required} available bytes including its safety reserve, but only {available} are available"
        );
    }
    Ok(())
}

pub(crate) fn available_bytes(path: &Path) -> Result<u64> {
    use std::os::unix::ffi::OsStrExt;

    let path = CString::new(path.as_os_str().as_bytes())
        .with_context(|| format!("Filesystem path contains a NUL byte: {}", path.display()))?;
    let mut stats = MaybeUninit::<libc::statvfs>::uninit();
    // SAFETY: `path` is a valid NUL-terminated C string and `stats` points to
    // writable storage for one `statvfs` value.
    if unsafe { libc::statvfs(path.as_ptr(), stats.as_mut_ptr()) } != 0 {
        return Err(Error::last_os_error()).context("Failed to inspect free space for shared import");
    }
    // SAFETY: a successful `statvfs` call initialized the output structure.
    let stats = unsafe { stats.assume_init() };
    stats.f_bavail.checked_mul(stats.f_frsize).context("Available filesystem byte count overflow")
}
