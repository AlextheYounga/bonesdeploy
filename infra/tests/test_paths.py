"""Paths manifest must define `build_logs` and `LOGS_DIR` for centralized log handling."""

from . import helpers


def test_paths_has_build_logs_constant():
    c = helpers.read(helpers.REPO_ROOT / "crates/shared/src/paths.rs")
    helpers.assert_contains(c, 'pub const LOGS_DIR: &str = "logs";')
    helpers.assert_contains(c, "pub build_logs: String,")
    helpers.assert_contains(c, "build_logs: Path::new(&project_root).join(BUILD_DIR).join(LOGS_DIR).display().to_string()")
