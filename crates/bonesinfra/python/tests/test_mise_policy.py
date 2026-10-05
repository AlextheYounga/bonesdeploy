from pathlib import Path

from bonesinfra.config.paths import SCRIPTS_DIR
from bonesinfra.services.languages.mise import (
    MISE_BINARY_SHA256,
    MISE_BINARY_URL,
    MISE_CACHE_DIR,
    MISE_CONFIG_DIR,
    MISE_DATA_DIR,
    MISE_RUNTIME_STORE,
    MISE_VERSION,
    PRODUCTION_MISE,
)


def test_production_mise_policy_pins_and_verifies_the_linux_binary():
    assert MISE_VERSION == "2026.10.0"
    assert MISE_BINARY_URL.endswith("mise-v2026.10.0-linux-x64")
    assert len(MISE_BINARY_SHA256) == 64
    assert MISE_DATA_DIR == "/var/lib/bonesdeploy/mise"
    assert MISE_RUNTIME_STORE == "/var/lib/bonesdeploy/mise/installs"
    assert MISE_CACHE_DIR == "/var/cache/bonesdeploy/mise"
    assert MISE_CONFIG_DIR == "/etc/bonesdeploy/mise"
    assert PRODUCTION_MISE.binary_url == MISE_BINARY_URL
    assert PRODUCTION_MISE.binary_sha256 == MISE_BINARY_SHA256


def test_production_mise_environment_is_precompiled_and_operator_owned():
    environment = PRODUCTION_MISE.environment()

    assert environment["MISE_SYSTEM_DATA_DIR"] == MISE_DATA_DIR
    assert environment["MISE_SYSTEM_INSTALLS_DIR"] == MISE_RUNTIME_STORE
    assert environment["MISE_CACHE_DIR"] == MISE_CACHE_DIR
    assert environment["MISE_GLOBAL_CONFIG_ROOT"] == MISE_CONFIG_DIR
    assert environment["MISE_CONFIG_FILE"] == f"{MISE_CONFIG_DIR}/config.toml"
    assert environment["MISE_ALL_COMPILE"] == "false"
    assert environment["MISE_NODE_COMPILE"] == "false"
    assert environment["MISE_PYTHON_COMPILE"] == "false"
    assert environment["MISE_RUBY_COMPILE"] == "false"
    assert environment["MISE_REGISTRY_FLOATING"] == "false"
    assert environment["MISE_AUTO_UPDATE"] == "false"
    assert environment["MISE_NO_HOOKS"] == "true"
    assert environment["MISE_SAFE"] == "true"
    assert environment["MISE_OVERRIDE_CONFIG_FILENAMES"] == ".bonesdeploy-mise-disabled"
    assert environment["MISE_OVERRIDE_TOOL_VERSIONS_FILENAMES"] == "none"
    assert environment["MISE_SLSA"] == "true"
    assert environment["MISE_GITHUB_ATTESTATIONS"] == "true"


def test_installer_links_the_site_to_the_immutable_runtime_root():
    installer = Path(str(SCRIPTS_DIR / "install-mise-runtime.sh")).read_text()

    assert "export tool version link_path" in installer
    assert 'flock "$lock_file" bash -ceu' in installer
    assert 'runtime_root="$MISE_RUNTIME_STORE/$tool/$version"' in installer
    assert '[[ -x "$runtime_root/bin/$tool" ]]' in installer
    assert 'chown -R root:root "$runtime_root"' in installer
    assert 'chmod -R go-w "$runtime_root"' in installer
    assert 'ln -sfn "$runtime_root" "$temporary_link"' in installer
