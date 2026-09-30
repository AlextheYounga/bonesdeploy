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

#[cfg(test)]
mod tests {
    use std::fs::{self, File, Permissions};
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::path::Path;

    use super::super::source::BuildContext;
    use super::{package, package_compose};
    use anyhow::{Context, Result};
    use bonesdeploy_core::artifact::{ArtifactKind, ComposeImage};
    use flate2::read::GzDecoder;

    #[test]
    fn package_preserves_directories_executables_and_safe_symlinks_but_excludes_root_env_files() -> Result<()> {
        let context = tempfile::tempdir()?;
        fs::create_dir(context.path().join("nested"))?;
        fs::write(context.path().join("nested/run.sh"), "#!/bin/sh\n")?;
        fs::set_permissions(context.path().join("nested/run.sh"), Permissions::from_mode(0o755))?;
        fs::write(context.path().join(".env"), "SECRET=no")?;
        fs::write(context.path().join(".env.build"), "NODE_VERSION=24")?;
        symlink("run.sh", context.path().join("nested/current"))?;
        let build = BuildContext::from_tempdir(context, "a".repeat(40));

        let artifact = package("atlas", &build)?;
        let file = File::open(artifact.path())?;
        let decoder = GzDecoder::new(file);
        let mut archive = tar::Archive::new(decoder);
        let mut names = Vec::new();
        for entry in archive.entries()? {
            let entry = entry?;
            names.push((entry.path()?.into_owned(), entry.header().mode()?));
        }
        assert!(names.iter().any(|(path, mode)| path == Path::new("nested/run.sh") && mode & 0o111 != 0));
        assert!(names.iter().any(|(path, _)| path == Path::new("nested/current")));
        assert!(!names.iter().any(|(path, _)| path == Path::new(".env")));
        assert!(!names.iter().any(|(path, _)| path == Path::new(".env.build")));
        assert_eq!(artifact.manifest.artifact_length, fs::metadata(artifact.path())?.len());
        Ok(())
    }

    #[test]
    fn package_rejects_symlinks_that_escape_the_build_context() -> Result<()> {
        let context = tempfile::tempdir()?;
        symlink("../../outside", context.path().join("escape"))?;
        let build = BuildContext::from_tempdir(context, "a".repeat(40));

        let error = package("atlas", &build).err().context("escaping symlink must be rejected")?;

        assert!(error.to_string().contains("escapes the build context"));
        Ok(())
    }

    #[test]
    fn package_supports_long_nested_paths_and_relative_link_targets() -> Result<()> {
        let context = tempfile::tempdir()?;
        let component = "segment-with-a-realistic-name-".repeat(2);
        let nested = context.path().join(&component).join(&component).join(&component);
        fs::create_dir_all(&nested)?;
        fs::write(nested.join("application-output.txt"), "built")?;
        let target = Path::new("..").join(&component).join("application-output.txt");
        symlink(&target, nested.join("current-output"))?;
        let build = BuildContext::from_tempdir(context, "a".repeat(40));

        let artifact = package("atlas", &build)?;
        let decoder = GzDecoder::new(File::open(artifact.path())?);
        let mut archive = tar::Archive::new(decoder);
        let expected = Path::new(&component).join(&component).join(&component).join("current-output");
        let link = archive.entries()?.find_map(|entry| {
            let entry = entry.ok()?;
            (entry.path().ok()?.as_ref() == expected).then_some(entry)
        });
        let link = link.context("long nested symlink missing from artifact")?;
        assert_eq!(link.link_name()?.context("symlink target missing")?, target);
        Ok(())
    }

    #[test]
    fn package_compose_records_validated_image_inventory() -> Result<()> {
        let context = tempfile::tempdir()?;
        fs::write(context.path().join("compose.yaml"), "services: {}\n")?;
        let build = BuildContext::from_tempdir(context, "a".repeat(40));

        let artifact =
            package_compose("atlas", &build, vec![ComposeImage::new("atlas", "web".into(), &build.revision)?])?;

        assert!(matches!(artifact.manifest.kind, ArtifactKind::ComposeImages { .. }));
        Ok(())
    }
}
