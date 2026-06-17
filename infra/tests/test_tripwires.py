"""Files and directories that must NOT exist (removed in migrations)."""

from . import helpers

R = helpers.REPO_ROOT
SRC = R / "crates/bonesdeploy/src"


def test_old_embeds_runtimes_dir_is_removed():
    helpers.assert_file_not_exists(R / "crates/bonesdeploy/embeds/runtimes")


def test_old_embeds_kit_dir_is_removed():
    helpers.assert_file_not_exists(R / "crates/bonesdeploy/embeds/kit")


def test_old_operations_py_does_not_exist():
    for p in ("infra/src/operations.py", "infra/runtime/operations.py"):
        helpers.assert_file_not_exists(R / p)


def test_pyinfra_rs_is_deleted():
    helpers.assert_file_not_exists(SRC / "pyinfra.rs")


def test_main_rs_has_no_pyinfra_mod():
    c = helpers.read(SRC / "main.rs")
    helpers.assert_not_contains(c, "mod pyinfra;")


def test_no_managed_pyinfra_in_shared_paths():
    c = helpers.read(R / "crates/shared/src/paths.rs")
    helpers.assert_not_contains(c, "managed_pyinfra_venv_dir")
    helpers.assert_not_contains(c, "managed_pyinfra_binary")


def test_config_rs_no_deploy_constants():
    c = helpers.read(R / "crates/bonesdeploy/src/config.rs")
    helpers.assert_not_contains(c, "BONES_REMOTE_SSL_DEPLOY")
    helpers.assert_not_contains(c, "BONES_REMOTE_SETUP_DEPLOY")


def test_embedded_rs_no_removed_functions():
    c = helpers.read(R / "crates/bonesdeploy/src/embedded.rs")
    helpers.assert_not_contains(c, "struct Runtimes")
    helpers.assert_not_contains(c, "fn scaffold_runtime_template")
    helpers.assert_not_contains(c, "fn read_template_runtime_config")
    helpers.assert_not_contains(c, "fn available_templates")


def test_main_py_has_apply_handlers():
    c = helpers.read(helpers.INFRA_DIR / "main.py")
    helpers.assert_contains(c, "def cmd_setup_apply")
    helpers.assert_contains(c, "def cmd_runtime_apply")
    helpers.assert_contains(c, "def cmd_ssl_apply")


def test_main_py_has_no_unimplemented():
    c = helpers.read(helpers.INFRA_DIR / "main.py")
    helpers.assert_not_contains(c, "UnimplementedError")


def test_infra_has_pyinfra_runner():
    helpers.assert_file_exists(helpers.SRC_DIR / "pyinfra_runner.py")


def test_infra_has_paths():
    helpers.assert_file_exists(helpers.SRC_DIR / "paths.py")
