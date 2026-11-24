use bonesdeploy_core::names;
use bonesremote::inspection::platform::supports_production_host;

#[test]
fn production_policy_accepts_supported_distribution_version_boundaries() {
    for release in [
        "ID=debian\nVERSION_ID=12",
        "ID=debian\nVERSION_ID=13.1",
        "ID=ubuntu\nVERSION_ID=24.04",
        "ID=ubuntu\nVERSION_ID=24.04.1",
        "ID=ubuntu\nVERSION_ID=25.10",
        "ID=ubuntu\nVERSION_ID=\"24.04\"",
    ] {
        assert!(supports_production_host(release, names::X86_64), "expected {release} to be accepted");
    }
}

#[test]
fn production_policy_rejects_unsupported_architectures() {
    for architecture in ["aarch64", "arm", "x86"] {
        assert!(!supports_production_host("ID=debian\nVERSION_ID=12", architecture));
    }
}

#[test]
fn production_policy_rejects_unsupported_and_malformed_distributions() {
    for release in [
        "ID=debian\nVERSION_ID=11",
        "ID=ubuntu\nVERSION_ID=22.04",
        "ID=fedora\nVERSION_ID=42",
        "ID=ubuntu",
        "ID=Ubuntu\nVERSION_ID=24.04",
        "ID=debian\nVERSION_ID=12-bookworm",
    ] {
        assert!(!supports_production_host(release, names::X86_64), "unexpectedly accepted {release}");
    }
}
