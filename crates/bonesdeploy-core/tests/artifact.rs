use std::io::Cursor;

use anyhow::Result;
use bonesdeploy_core::artifact::{
    self, ArtifactKind, ArtifactManifest, ComposeImage, MAX_ARTIFACT_PAYLOAD_BYTES, MAX_COMPOSE_IMAGES,
};

const REVISION: &str = "0123456789abcdef0123456789abcdef01234567";

fn native_manifest() -> ArtifactManifest {
    ArtifactManifest::new_native_tree("atlas".into(), REVISION.into(), 42, &"a".repeat(64))
}

fn compose_manifest() -> Result<ArtifactManifest> {
    Ok(ArtifactManifest::new_native_tree("atlas".into(), REVISION.into(), 42, &"a".repeat(64))
        .with_compose_images(vec![ComposeImage::new("atlas", "web".into(), REVISION)?]))
}

#[test]
fn native_manifest_round_trips_through_strict_frame() -> Result<()> {
    let original = native_manifest();
    let json = serde_json::to_value(&original)?;
    assert_eq!(json["kind"], "native-tree");
    assert!(json.get("builder_image").is_none());

    let frame = artifact::encode_manifest(&original)?;
    assert_eq!(artifact::decode_manifest(&frame)?, original);

    let mut reader = Cursor::new(frame);
    assert_eq!(artifact::read_manifest(&mut reader)?, original);
    Ok(())
}

#[test]
fn compose_manifest_round_trips_and_contains_derived_image_tag() -> Result<()> {
    let original = compose_manifest()?;
    let json = serde_json::to_value(&original)?;
    assert_eq!(json["kind"], "compose-images");
    assert_eq!(json["metadata"]["images"][0]["service"], "web");
    assert_eq!(json["metadata"]["images"][0]["tag"], "bonesdeploy-atlas-web-0123456789abcdef0123456789abcdef01234567");

    let frame = artifact::encode_manifest(&original)?;
    assert_eq!(artifact::decode_manifest(&frame)?, original);
    Ok(())
}

#[test]
fn manifest_rejects_unknown_fields_and_wrong_protocol() {
    let unknown = serde_json::json!({
        "protocol_version": 2,
        "site": "atlas",
        "revision": REVISION,
        "target": "linux/amd64",
        "archive_format": "gzip-posix-tar",
        "artifact_length": 1,
        "sha256": "a".repeat(64),
        "kind": "native-tree",
        "extra": true
    });
    assert!(serde_json::from_value::<ArtifactManifest>(unknown).is_err());

    let native_metadata = serde_json::json!({
        "protocol_version": 2,
        "site": "atlas",
        "revision": REVISION,
        "target": "linux/amd64",
        "archive_format": "gzip-posix-tar",
        "artifact_length": 1,
        "sha256": "a".repeat(64),
        "kind": "native-tree",
        "metadata": {"images": []}
    });
    assert!(serde_json::from_value::<ArtifactManifest>(native_metadata).is_err());

    let mut wrong = native_manifest();
    wrong.protocol_version = 99;
    assert!(wrong.validate().is_err());
}

#[test]
fn manifest_rejects_invalid_site_revision_archive_format_and_payload_size() {
    let mut invalid = native_manifest();
    invalid.site = "../atlas".into();
    assert!(invalid.validate().is_err());

    invalid = native_manifest();
    invalid.revision = "deadbeef".into();
    assert!(invalid.validate().is_err());

    let wrong_format = serde_json::json!({
        "protocol_version": 2,
        "site": "atlas",
        "revision": REVISION,
        "target": "linux/amd64",
        "archive_format": "tar",
        "artifact_length": 1,
        "sha256": "a".repeat(64),
        "kind": "native-tree"
    });
    assert!(serde_json::from_value::<ArtifactManifest>(wrong_format).is_err());

    invalid = native_manifest();
    invalid.artifact_length = MAX_ARTIFACT_PAYLOAD_BYTES + 1;
    assert!(invalid.validate().is_err());

    invalid = native_manifest();
    invalid.artifact_length = 0;
    assert!(invalid.validate().is_err());
}

#[test]
fn compose_inventory_rejects_wrong_tags_duplicates_empty_inventory_and_bad_services() -> Result<()> {
    let mut invalid = compose_manifest()?;
    let ArtifactKind::ComposeImages { images } = &mut invalid.kind else { unreachable!() };
    images[0].tag = "latest".into();
    assert!(invalid.validate().is_err());

    let mut invalid = compose_manifest()?;
    let ArtifactKind::ComposeImages { images } = &mut invalid.kind else { unreachable!() };
    images.push(ComposeImage::new("atlas", "web-2".into(), REVISION)?);
    images[1].service = "web".into();
    images[1].tag = artifact::compose_image_tag("atlas", "web", REVISION)?;
    assert!(invalid.validate().is_err());

    let mut invalid = native_manifest();
    invalid.kind = ArtifactKind::ComposeImages { images: Vec::new() };
    assert!(invalid.validate().is_err());

    assert!(ComposeImage::new("atlas", "Web".into(), REVISION).is_err());
    assert!(artifact::compose_image_tag("atlas", "-web", REVISION).is_err());
    Ok(())
}

#[test]
fn compose_inventory_is_bounded() -> Result<()> {
    let mut images = Vec::new();
    for index in 0..=MAX_COMPOSE_IMAGES {
        images.push(ComposeImage::new("atlas", format!("web-{index}"), REVISION)?);
    }
    let manifest = ArtifactManifest::new_native_tree("atlas".into(), REVISION.into(), 42, &"a".repeat(64))
        .with_compose_images(images);
    assert!(manifest.validate().is_err());
    Ok(())
}

#[test]
fn manifest_constructor_normalizes_sha256_to_lowercase() {
    let manifest = ArtifactManifest::new_native_tree("atlas".into(), REVISION.into(), 1, &"A".repeat(64));
    assert_eq!(manifest.sha256, "a".repeat(64));
}

#[test]
fn manifest_frame_rejects_truncation_and_oversized_header() -> Result<()> {
    let frame = artifact::encode_manifest(&native_manifest())?;
    assert!(artifact::decode_manifest(&frame[..frame.len() - 1]).is_err());

    let oversized = (65_536_u32 + 1).to_be_bytes();
    assert!(artifact::decode_manifest(&oversized).is_err());
    Ok(())
}
