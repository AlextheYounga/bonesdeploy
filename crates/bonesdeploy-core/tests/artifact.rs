use std::io::Cursor;

use anyhow::Result;
use bonesdeploy_core::artifact::{self, ArtifactManifest};

fn manifest() -> ArtifactManifest {
    ArtifactManifest::new("atlas".into(), "0123456789abcdef0123456789abcdef01234567".into(), 42, &"a".repeat(64))
}

#[test]
fn manifest_round_trips_through_strict_frame() -> Result<()> {
    let original = manifest();
    let json = serde_json::to_value(&original)?;
    assert_eq!(json["target"], "linux/amd64");
    let frame = artifact::encode_manifest(&original)?;
    assert_eq!(artifact::decode_manifest(&frame)?, original);

    let mut reader = Cursor::new(frame);
    assert_eq!(artifact::read_manifest(&mut reader)?, original);
    Ok(())
}

#[test]
fn manifest_rejects_unknown_fields_and_wrong_protocol() {
    let unknown = r#"{"protocol_version":1,"site":"atlas","revision":"0123456789abcdef0123456789abcdef01234567","target":"linux/amd64","builder_image":"sha256:5ac8377b7884040464fbf9390409073c6ac62ac8e6563e886fc560f9ea13ec5d","archive_format":"gzip-posix-tar","artifact_length":1,"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","extra":true}"#;
    assert!(serde_json::from_str::<ArtifactManifest>(unknown).is_err());

    let mut wrong = manifest();
    wrong.protocol_version = 99;
    assert!(wrong.validate().is_err());
}

#[test]
fn manifest_rejects_invalid_site_revision_and_archive_format() {
    let mut invalid = manifest();
    invalid.site = "../atlas".into();
    assert!(invalid.validate().is_err());

    invalid = manifest();
    invalid.revision = "deadbeef".into();
    assert!(invalid.validate().is_err());

    let wrong_format = serde_json::json!({
        "protocol_version": 1,
        "site": "atlas",
        "revision": "0123456789abcdef0123456789abcdef01234567",
        "target": "linux/amd64",
        "builder_image": "sha256:5ac8377b7884040464fbf9390409073c6ac62ac8e6563e886fc560f9ea13ec5d",
        "archive_format": "tar",
        "artifact_length": 1,
        "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    });
    assert!(serde_json::from_value::<ArtifactManifest>(wrong_format).is_err());

    let mut wrong_builder = manifest();
    wrong_builder.builder_image = "sha256:wrong".into();
    assert!(wrong_builder.validate().is_err());

    let wrong_target = serde_json::json!({
        "protocol_version": 1,
        "site": "atlas",
        "revision": "0123456789abcdef0123456789abcdef01234567",
        "target": "linux/arm64",
        "builder_image": "sha256:5ac8377b7884040464fbf9390409073c6ac62ac8e6563e886fc560f9ea13ec5d",
        "archive_format": "gzip-posix-tar",
        "artifact_length": 1,
        "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    });
    assert!(serde_json::from_value::<ArtifactManifest>(wrong_target).is_err());

    let wrong_target_type = serde_json::json!({
        "protocol_version": 1,
        "site": "atlas",
        "revision": "0123456789abcdef0123456789abcdef01234567",
        "target": null,
        "builder_image": "sha256:5ac8377b7884040464fbf9390409073c6ac62ac8e6563e886fc560f9ea13ec5d",
        "archive_format": "gzip-posix-tar",
        "artifact_length": 1,
        "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    });
    assert!(serde_json::from_value::<ArtifactManifest>(wrong_target_type).is_err());
}

#[test]
fn manifest_constructor_normalizes_sha256_to_lowercase() {
    let manifest =
        ArtifactManifest::new("atlas".into(), "0123456789abcdef0123456789abcdef01234567".into(), 1, &"A".repeat(64));
    assert_eq!(manifest.sha256, "a".repeat(64));
}

#[test]
fn manifest_frame_rejects_truncation_and_oversized_header() -> Result<()> {
    let frame = artifact::encode_manifest(&manifest())?;
    assert!(artifact::decode_manifest(&frame[..frame.len() - 1]).is_err());

    let oversized = (16_384_u32 + 1).to_be_bytes();
    assert!(artifact::decode_manifest(&oversized).is_err());
    Ok(())
}
