//! Strict artifact manifest and length-prefixed protocol framing.

use std::io::{self, Read, Write};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::build_contract::TargetPlatform;
use crate::config::validate_site_name;

pub const ARTIFACT_PROTOCOL_VERSION: u16 = 2;
pub const MAX_MANIFEST_BYTES: usize = 64 * 1024;
pub const MAX_ARTIFACT_PAYLOAD_BYTES: u64 = 2 * 1024 * 1024 * 1024;
pub const MAX_COMPOSE_IMAGES: usize = 128;
pub const MAX_COMPOSE_SERVICE_NAME_BYTES: usize = 64;
pub const MAX_COMPOSE_TAG_BYTES: usize = 128;
pub const FRAME_LENGTH_BYTES: usize = 4;
pub const COMPOSE_OVERRIDE_FILE: &str = ".bonesdeploy-compose.override.yaml";
pub const COMPOSE_IMAGE_INVENTORY_FILE: &str = ".bonesdeploy-compose-images.json";
pub const COMPOSE_IMAGE_ARCHIVE_FILE: &str = ".bonesdeploy-compose-images.tar";

const GIT_OBJECT_ID_LENGTH: usize = 40;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArchiveFormat {
    GzipPosixTar,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComposeImage {
    pub service: String,
    pub tag: String,
}

impl ComposeImage {
    /// Creates the release-specific tag owned by this artifact.
    pub fn new(site: &str, service: String, revision: &str) -> Result<Self> {
        validate_site_name(site)?;
        validate_revision(revision)?;
        validate_compose_service_name(&service)?;
        Ok(Self { tag: compose_image_tag(site, &service, revision)?, service })
    }
}

/// Returns the generated Compose override carried by a Compose artifact.
#[must_use]
pub fn compose_override_contents(images: &[ComposeImage]) -> String {
    let mut contents = String::from("services:\n");
    for image in images {
        contents.push_str(&format!("  {}:\n    image: {}\n", image.service, image.tag));
    }
    contents
}

/// Metadata describing the payload layout and any Compose images it contains.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "metadata", rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub enum ArtifactKind {
    NativeTree,
    ComposeImages { images: Vec<ComposeImage> },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactManifest {
    pub protocol_version: u16,
    pub site: String,
    pub revision: String,
    pub target: TargetPlatform,
    pub archive_format: ArchiveFormat,
    pub artifact_length: u64,
    pub sha256: String,
    #[serde(flatten)]
    pub kind: ArtifactKind,
}

impl ArtifactManifest {
    /// Creates a native filesystem-tree manifest.
    #[must_use]
    pub fn new(site: String, revision: String, artifact_length: u64, sha256: &str) -> Self {
        Self::new_native_tree(site, revision, artifact_length, sha256)
    }

    #[must_use]
    pub fn new_native_tree(site: String, revision: String, artifact_length: u64, sha256: &str) -> Self {
        Self {
            protocol_version: ARTIFACT_PROTOCOL_VERSION,
            site,
            revision,
            target: TargetPlatform::LinuxAmd64,
            archive_format: ArchiveFormat::GzipPosixTar,
            artifact_length,
            sha256: sha256.to_ascii_lowercase(),
            kind: ArtifactKind::NativeTree,
        }
    }

    #[must_use]
    pub fn with_compose_images(mut self, images: Vec<ComposeImage>) -> Self {
        self.kind = ArtifactKind::ComposeImages { images };
        self
    }

    /// Validates all facts that the artifact receiver can enforce.
    pub fn validate(&self) -> Result<()> {
        if self.protocol_version != ARTIFACT_PROTOCOL_VERSION {
            bail!("unsupported artifact protocol version {}", self.protocol_version);
        }
        validate_site_name(&self.site)?;
        validate_revision(&self.revision)?;
        if self.archive_format != ArchiveFormat::GzipPosixTar {
            bail!("unsupported artifact archive format");
        }
        if self.artifact_length == 0 || self.artifact_length > MAX_ARTIFACT_PAYLOAD_BYTES {
            bail!("artifact payload must be between 1 and {MAX_ARTIFACT_PAYLOAD_BYTES} bytes");
        }
        if self.sha256.len() != 64 || !self.sha256.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            bail!("artifact sha256 must be 64 hexadecimal characters");
        }
        if self.sha256.bytes().any(|byte| byte.is_ascii_uppercase()) {
            bail!("artifact sha256 must use lowercase hexadecimal");
        }
        match &self.kind {
            ArtifactKind::NativeTree => Ok(()),
            ArtifactKind::ComposeImages { images } => validate_images(&self.site, &self.revision, images),
        }
    }
}

fn validate_revision(revision: &str) -> Result<()> {
    if revision.len() != GIT_OBJECT_ID_LENGTH || !revision.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("artifact revision must be a full 40-character Git object id");
    }
    if revision.bytes().any(|byte| byte.is_ascii_uppercase()) {
        bail!("artifact revision must use lowercase hexadecimal");
    }
    Ok(())
}

fn validate_images(site: &str, revision: &str, images: &[ComposeImage]) -> Result<()> {
    if images.is_empty() {
        bail!("Compose artifact image inventory cannot be empty");
    }
    if images.len() > MAX_COMPOSE_IMAGES {
        bail!("Compose artifact contains more than {MAX_COMPOSE_IMAGES} images");
    }
    for (index, image) in images.iter().enumerate() {
        validate_compose_service_name(&image.service)?;
        let expected_tag = compose_image_tag(site, &image.service, revision)?;
        if image.tag != expected_tag {
            bail!("Compose image {index} has a tag that does not match its site, service, and revision");
        }
        if images[..index].iter().any(|previous| previous.service == image.service) {
            bail!("Compose artifact contains duplicate service `{}`", image.service);
        }
    }
    Ok(())
}

fn validate_compose_service_name(service: &str) -> Result<()> {
    if service.is_empty()
        || service.len() > MAX_COMPOSE_SERVICE_NAME_BYTES
        || !service
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-')
        || !service.bytes().next().is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
    {
        bail!("invalid Compose service name `{service}`");
    }
    Ok(())
}

/// Returns the immutable Docker tag for one service in a release.
pub fn compose_image_tag(site: &str, service: &str, revision: &str) -> Result<String> {
    validate_site_name(site)?;
    validate_revision(revision)?;
    validate_compose_service_name(service)?;
    let tag = format!("bonesdeploy-{site}-{service}-{revision}");
    if tag.len() > MAX_COMPOSE_TAG_BYTES {
        bail!("Compose image tag exceeds {MAX_COMPOSE_TAG_BYTES} bytes");
    }
    Ok(tag)
}

/// Encodes the strict JSON manifest preceded by its big-endian byte length.
pub fn encode_manifest(manifest: &ArtifactManifest) -> Result<Vec<u8>> {
    manifest.validate()?;
    let json = serde_json::to_vec(manifest).context("failed to serialize artifact manifest")?;
    if json.len() > MAX_MANIFEST_BYTES {
        bail!("artifact manifest exceeds {} bytes", MAX_MANIFEST_BYTES);
    }
    let length = u32::try_from(json.len()).context("artifact manifest is too large to frame")?;
    let mut frame = Vec::with_capacity(FRAME_LENGTH_BYTES + json.len());
    frame.extend_from_slice(&length.to_be_bytes());
    frame.extend_from_slice(&json);
    Ok(frame)
}

/// Decodes a complete framed manifest header. Artifact bytes are not consumed.
pub fn decode_manifest(frame: &[u8]) -> Result<ArtifactManifest> {
    if frame.len() < FRAME_LENGTH_BYTES {
        bail!("truncated artifact manifest length header");
    }
    let length = u32::from_be_bytes(
        frame[..FRAME_LENGTH_BYTES].try_into().map_err(|_| anyhow::anyhow!("invalid manifest length header"))?,
    ) as usize;
    if length > MAX_MANIFEST_BYTES {
        bail!("artifact manifest exceeds {} bytes", MAX_MANIFEST_BYTES);
    }
    if frame.len() != FRAME_LENGTH_BYTES + length {
        bail!("artifact manifest frame is truncated or has trailing bytes");
    }
    let manifest: ArtifactManifest =
        serde_json::from_slice(&frame[FRAME_LENGTH_BYTES..]).context("failed to parse artifact manifest")?;
    manifest.validate()?;
    Ok(manifest)
}

/// Writes a manifest frame for callers that already have an async transport's
/// blocking adapter or a small in-memory protocol buffer.
pub fn write_manifest<W: Write>(writer: &mut W, manifest: &ArtifactManifest) -> Result<()> {
    writer.write_all(&encode_manifest(manifest)?).context("failed to write artifact manifest")?;
    Ok(())
}

/// Reads exactly one framed manifest without reading artifact payload bytes.
pub fn read_manifest<R: Read>(reader: &mut R) -> Result<ArtifactManifest> {
    let mut length = [0; FRAME_LENGTH_BYTES];
    reader.read_exact(&mut length).context("failed to read artifact manifest length")?;
    let size = u32::from_be_bytes(length) as usize;
    if size > MAX_MANIFEST_BYTES {
        bail!("artifact manifest exceeds {} bytes", MAX_MANIFEST_BYTES);
    }
    let mut frame = Vec::with_capacity(FRAME_LENGTH_BYTES + size);
    frame.extend_from_slice(&length);
    frame.resize(FRAME_LENGTH_BYTES + size, 0);
    reader.read_exact(&mut frame[FRAME_LENGTH_BYTES..]).map_err(|error| match error.kind() {
        io::ErrorKind::UnexpectedEof => anyhow::anyhow!("truncated artifact manifest"),
        _ => anyhow::Error::from(error),
    })?;
    decode_manifest(&frame)
}
