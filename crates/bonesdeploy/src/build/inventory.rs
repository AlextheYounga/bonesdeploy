use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};
use std::str;

use anyhow::{Result, bail};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntryType {
    File,
    Directory,
    Symlink,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Entry {
    pub entry_type: EntryType,
    pub mode: u32,
}

#[derive(Default)]
pub struct BuildInventory {
    entries: BTreeMap<String, Entry>,
}

impl BuildInventory {
    pub fn record(&mut self, path: &Path, entry_type: EntryType, mode: u32) -> Result<()> {
        let path = canonical_relative_path(path)?;
        if mode > 0o7777 {
            bail!("Build inventory entry {path} has invalid Unix mode {mode:o}");
        }
        if self.entries.insert(path.clone(), Entry { entry_type, mode }).is_some() {
            bail!("Build inventory contains duplicate path {path}");
        }
        Ok(())
    }

    pub fn get(&self, path: &Path) -> Option<Entry> {
        canonical_relative_path(path).ok().and_then(|path| self.entries.get(&path).copied())
    }

    pub fn from_builder_output(output: &[u8]) -> Result<Self> {
        let fields: Vec<_> = output.split(|byte| *byte == b'\0').collect();
        if fields.last() != Some(&&[][..]) || fields.len() % 3 != 1 {
            bail!("Linux builder returned an invalid build inventory");
        }

        let mut inventory = Self::default();
        for fields in fields[..fields.len() - 1].chunks_exact(3) {
            let path =
                str::from_utf8(fields[0]).map_err(|_| anyhow::anyhow!("Linux builder inventory path is not UTF-8"))?;
            let entry_type = match fields[1] {
                b"f" => EntryType::File,
                b"d" => EntryType::Directory,
                b"l" => EntryType::Symlink,
                _ => bail!("Linux builder inventory has an unsupported entry type"),
            };
            let mode = u32::from_str_radix(
                str::from_utf8(fields[2]).map_err(|_| anyhow::anyhow!("Linux builder inventory mode is not UTF-8"))?,
                8,
            )
            .map_err(|_| anyhow::anyhow!("Linux builder inventory mode is invalid"))?;
            inventory.record(Path::new(path), entry_type, mode)?;
        }
        Ok(inventory)
    }
}

pub fn validate_path(path: &Path) -> Result<()> {
    canonical_relative_path(path).map(|_| ())
}

pub fn canonical_relative_path(path: &Path) -> Result<String> {
    if path.as_os_str().is_empty() || path.is_absolute() || path.to_str().is_none() {
        bail!("Build inventory path must be a non-empty UTF-8 relative path: {}", path.display());
    }
    let mut canonical = String::new();
    for component in path.components() {
        let Component::Normal(component) = component else {
            bail!("Build inventory path is not relative and normalized: {}", path.display());
        };
        let component = component
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Build inventory path must be UTF-8: {}", path.display()))?;
        if component.contains('\\') {
            bail!("Build inventory path must not contain a backslash: {}", path.display());
        }
        if !canonical.is_empty() {
            canonical.push('/');
        }
        canonical.push_str(component);
    }
    Ok(canonical)
}

pub fn safe_relative_link(path: &Path, target: &Path) -> bool {
    if target.is_absolute() || target.to_str().is_none() || target.to_string_lossy().contains('\\') {
        return false;
    }
    let mut resolved = PathBuf::new();
    for component in path.parent().unwrap_or(Path::new("")).join(target).components() {
        match component {
            Component::Normal(part) => resolved.push(part),
            Component::ParentDir if !resolved.pop() => return false,
            Component::ParentDir | Component::CurDir => {}
            Component::RootDir | Component::Prefix(_) => return false,
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{BuildInventory, EntryType, canonical_relative_path};

    #[test]
    fn builder_inventory_rejects_invalid_paths_types_and_modes() {
        assert!(BuildInventory::from_builder_output(b"../escape\0f\x00755\0").is_err());
        assert!(BuildInventory::from_builder_output(b"file\0x\x00755\0").is_err());
        assert!(BuildInventory::from_builder_output(b"file\0f\0invalid\0").is_err());
    }

    #[test]
    fn builder_inventory_parses_null_delimited_entries() -> anyhow::Result<()> {
        let inventory = BuildInventory::from_builder_output(b"bin\0d\x00755\0bin/run\0f\x00755\0")?;
        assert_eq!(
            inventory.get(Path::new("bin/run")),
            Some(super::Entry { entry_type: EntryType::File, mode: 0o755 })
        );
        Ok(())
    }

    #[test]
    fn inventory_paths_use_slashes_for_archive_and_lookup_keys() -> anyhow::Result<()> {
        let path = Path::new("nested").join("run.sh");
        let mut inventory = BuildInventory::default();
        inventory.record(&path, EntryType::File, 0o755)?;

        assert_eq!(canonical_relative_path(&path)?, "nested/run.sh");
        assert_eq!(inventory.get(&path), Some(super::Entry { entry_type: EntryType::File, mode: 0o755 }));
        Ok(())
    }
}
