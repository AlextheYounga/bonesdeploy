"""Files and directories that must NOT exist (removed in migrations)."""

from . import helpers

R = helpers.REPO_ROOT


def test_old_embeds_runtimes_dir_is_removed():
    helpers.assert_file_not_exists(R / "crates/bonesdeploy/embeds/runtimes")


def test_old_embeds_kit_dir_is_removed():
    helpers.assert_file_not_exists(R / "crates/bonesdeploy/embeds/kit")


def test_old_operations_py_does_not_exist():
    for p in ("infra/src/operations.py", "infra/runtime/operations.py"):
        helpers.assert_file_not_exists(R / p)


def test_embedded_rs_no_removed_functions():
    c = helpers.read(R / "crates/bonesdeploy/src/embedded.rs")
    helpers.assert_not_contains(c, "struct Runtimes")
    helpers.assert_not_contains(c, "fn scaffold_runtime_template")
    helpers.assert_not_contains(c, "fn read_template_runtime_config")
    helpers.assert_not_contains(c, "fn available_templates")
