use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, bail};
use bonesdeploy_core::artifact::{ArtifactManifest, ComposeImage};
use bonesdeploy_core::paths;
use flate2::Compression;
use flate2::write::GzEncoder;
use sha2::{Digest, Sha256};
use tempfile::NamedTempFile;

use super::source::BuildContext;

pub struct PackagedArtifact {
    file: NamedTempFile,
    pub manifest: ArtifactManifest,
}

impl PackagedArtifact {
    pub fn path(&self) -> &Path {
        self.file.path()
    }
}

pub fn package(site: &str, build: &BuildContext) -> Result<PackagedArtifact> {
    let file = tempfile::Builder::new()
        .prefix("bonesdeploy-artifact-")
        .tempfile()
        .context("Failed to create artifact file")?;
    fs::set_permissions(file.path(), fs::Permissions::from_mode(0o600))?;
    {
        let encoder = GzEncoder::new(file.reopen().context("Failed to open artifact file")?, Compression::default());
        let mut archive = tar::Builder::new(encoder);
        append_tree(&mut archive, build.path(), Path::new(""))?;
        archive.finish().context("Failed to finish artifact archive")?;
        archive
            .into_inner()
            .context("Failed to finish artifact compression")?
            .finish()
            .context("Failed to finish artifact file")?;
    }
    let (length, digest) = digest(file.path())?;
    Ok(PackagedArtifact {
        file,
        manifest: ArtifactManifest::new(site.to_string(), build.revision.clone(), length, &digest),
    })
}

pub fn package_compose(site: &str, build: &BuildContext, images: Vec<ComposeImage>) -> Result<PackagedArtifact> {
    let mut artifact = package(site, build)?;
    artifact.manifest = artifact.manifest.with_compose_images(images);
    artifact.manifest.validate()?;
    Ok(artifact)
}

fn append_tree<W: Write>(archive: &mut tar::Builder<W>, root: &Path, relative: &Path) -> Result<()> {
    for entry in fs::read_dir(root.join(relative))
        .with_context(|| format!("Failed to read build context {}", relative.display()))?
    {
        let entry = entry?;
        let name = entry.file_name();
        let child = relative.join(&name);
        if child == Path::new(paths::DOT_ENV) || child == Path::new(paths::ENV_BUILD_FILE) {
            continue;
        }
        let source = root.join(&child);
        let metadata = fs::symlink_metadata(&source)
            .with_context(|| format!("Failed to inspect artifact entry {}", child.display()))?;
        if metadata.file_type().is_dir() {
            append_directory(archive, &source, &child, &metadata)?;
            append_tree(archive, root, &child)?;
        } else if metadata.file_type().is_file() {
            append_file(archive, &source, &child, &metadata)?;
        } else if metadata.file_type().is_symlink() {
            let target =
                fs::read_link(&source).with_context(|| format!("Failed to read symlink {}", child.display()))?;
            if !safe_relative_link(&child, &target) {
                bail!("Artifact symlink {} escapes the build context", child.display());
            }
            append_symlink(archive, &child, &target, &metadata)?;
        } else {
            bail!("Artifact entry {} has an unsupported file type", child.display());
        }
    }
    Ok(())
}

fn append_directory<W: Write>(
    archive: &mut tar::Builder<W>,
    source: &Path,
    path: &Path,
    metadata: &fs::Metadata,
) -> Result<()> {
    let mut header = tar::Header::new_ustar();
    header.set_metadata_in_mode(metadata, tar::HeaderMode::Deterministic);
    header.set_entry_type(tar::EntryType::Directory);
    header.set_size(0);
    header.set_cksum();
    archive
        .append_data(&mut header, path, io::empty())
        .with_context(|| format!("Failed to add directory {}", source.display()))
}

fn append_file<W: Write>(
    archive: &mut tar::Builder<W>,
    source: &Path,
    path: &Path,
    metadata: &fs::Metadata,
) -> Result<()> {
    let mut header = tar::Header::new_ustar();
    header.set_metadata_in_mode(metadata, tar::HeaderMode::Deterministic);
    header.set_size(metadata.len());
    header.set_cksum();
    let file = File::open(source).with_context(|| format!("Failed to read artifact file {}", source.display()))?;
    archive
        .append_data(&mut header, path, file)
        .with_context(|| format!("Failed to add artifact file {}", source.display()))
}

fn append_symlink<W: Write>(
    archive: &mut tar::Builder<W>,
    path: &Path,
    target: &Path,
    metadata: &fs::Metadata,
) -> Result<()> {
    let mut header = tar::Header::new_ustar();
    header.set_metadata_in_mode(metadata, tar::HeaderMode::Deterministic);
    header.set_entry_type(tar::EntryType::Symlink);
    header.set_size(0);
    header.set_link_name(target).context("Failed to set artifact symlink target")?;
    header.set_cksum();
    archive
        .append_data(&mut header, path, io::empty())
        .with_context(|| format!("Failed to add artifact symlink {}", path.display()))
}

fn safe_relative_link(path: &Path, target: &Path) -> bool {
    if target.is_absolute() {
        return false;
    }
    let mut resolved = PathBuf::new();
    for component in path.parent().unwrap_or(Path::new("")).join(target).components() {
        match component {
            Component::Normal(part) => resolved.push(part),
            Component::ParentDir => {
                if !resolved.pop() {
                    return false;
                }
            }
            Component::CurDir => {}
            Component::RootDir | Component::Prefix(_) => return false,
        }
    }
    true
}

fn digest(path: &Path) -> Result<(u64, String)> {
    let mut file = File::open(path).with_context(|| format!("Failed to read artifact {}", path.display()))?;
    let mut hash = Sha256::new();
    let mut length = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer).context("Failed to hash artifact")?;
        if read == 0 {
            break;
        }
        length += u64::try_from(read).context("Artifact exceeds supported length")?;
        hash.update(&buffer[..read]);
    }
    Ok((length, format!("{:x}", hash.finalize())))
}
