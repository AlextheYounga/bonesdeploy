//! Strict artifact manifest and length-prefixed protocol framing.

use std::io::{self, Read, Write};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::build_contract::{BUILDER_IMAGE_DIGEST, TargetPlatform};
use crate::config::validate_site_name;

pub const ARTIFACT_PROTOCOL_VERSION: u16 = 1;
pub const MAX_MANIFEST_BYTES: usize = 16 * 1024;
pub const FRAME_LENGTH_BYTES: usize = 4;

const GIT_OBJECT_ID_LENGTH: usize = 40;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArchiveFormat {
    GzipPosixTar,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactManifest {
    pub protocol_version: u16,
    pub site: String,
    pub revision: String,
    pub target: TargetPlatform,
    pub builder_image: String,
    pub archive_format: ArchiveFormat,
    pub artifact_length: u64,
    pub sha256: String,
}

impl ArtifactManifest {
    #[must_use]
    pub fn new(site: String, revision: String, artifact_length: u64, sha256: &str) -> Self {
        Self {
            protocol_version: ARTIFACT_PROTOCOL_VERSION,
            site,
            revision,
            target: TargetPlatform::LinuxAmd64,
            builder_image: BUILDER_IMAGE_DIGEST.to_string(),
            archive_format: ArchiveFormat::GzipPosixTar,
            artifact_length,
            sha256: sha256.to_ascii_lowercase(),
        }
    }

    /// Validates protocol and builder identity before an artifact is received.
    pub fn validate(&self) -> Result<()> {
        if self.protocol_version != ARTIFACT_PROTOCOL_VERSION {
            bail!("unsupported artifact protocol version {}", self.protocol_version);
        }
        validate_site_name(&self.site)?;
        if self.revision.len() != GIT_OBJECT_ID_LENGTH || !self.revision.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            bail!("artifact revision must be a full 40-character Git object id");
        }
        if self.revision.bytes().any(|byte| byte.is_ascii_uppercase()) {
            bail!("artifact revision must use lowercase hexadecimal");
        }
        if self.builder_image != BUILDER_IMAGE_DIGEST {
            bail!("unsupported artifact builder image `{}`", self.builder_image);
        }
        if self.archive_format != ArchiveFormat::GzipPosixTar {
            bail!("unsupported artifact archive format");
        }
        if self.sha256.len() != 64 || !self.sha256.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            bail!("artifact sha256 must be 64 hexadecimal characters");
        }
        Ok(())
    }
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
