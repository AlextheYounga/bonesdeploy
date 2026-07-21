"""Files and directories that must NOT exist (removed in migrations)."""

from . import helpers

R = helpers.REPO_ROOT
SRC = R / "crates/bonesdeploy/src"
CRATES_EXIST = R.joinpath("crates").is_dir()


def test_old_embeds_runtimes_dir_is_removed():
    helpers.assert_file_not_exists(R / "crates/bonesdeploy/embeds/runtimes")


def test_old_embeds_kit_dir_is_removed():
    helpers.assert_file_not_exists(R / "crates/bonesdeploy/embeds/kit")


def test_old_operations_py_does_not_exist():
    for p in ("infra/src/operations.py", "infra/runtime/operations.py"):
        helpers.assert_file_not_exists(R / p)


def test_pyinfra_rs_is_deleted():
    helpers.assert_file_not_exists(SRC / "pyinfra.rs")


def test_cli_has_apply_handlers():
    c = helpers.read(helpers.SRC_DIR / "bonesinfra/cli/app.py")
    helpers.assert_contains(c, "setup_apply_cmd")
    helpers.assert_contains(c, "runtime_apply_cmd")
    helpers.assert_contains(c, "ssl_apply_cmd")


def test_infra_has_pyinfra_runner():
    helpers.assert_file_exists(helpers.SRC_DIR / "bonesinfra/infra/pyinfra_runner.py")


def test_infra_has_paths():
    helpers.assert_file_exists(helpers.SRC_DIR / "bonesinfra/domain/paths.py")
