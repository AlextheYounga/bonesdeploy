"""CLI commands must run without crashing."""

from . import helpers


MAIN = helpers.INFRA_DIR / "main.py"


def test_runtime_list():
    helpers.run(MAIN, "runtime", "list", "--json")


def test_runtime_questions_all():
    for name in ["django", "laravel", "next", "rails", "sveltekit", "vue"]:
        helpers.run(MAIN, "runtime", "questions", name, "--json")


def test_runtime_defaults():
    for name in ["django", "laravel", "next"]:
        helpers.run(MAIN, "runtime", "defaults", name, "--json")


def test_unimplemented_commands_fail_gracefully():
    import subprocess

    for args in (
        ["setup", "apply", "--config", "/dev/null"],
        ["runtime-apply", "apply", "--config", "/dev/null", "--runtime-config", "/dev/null"],
        ["ssl", "apply", "--config", "/dev/null"],
    ):
        result = subprocess.run(
            ["python3", str(MAIN), *args],
            capture_output=True,
            text=True,
            timeout=10,
        )
        assert result.returncode == 2, f"Expected exit 2 for unimplemented: {' '.join(args)}"
        assert "Error" in result.stderr, f"Expected error message: {' '.join(args)}"
