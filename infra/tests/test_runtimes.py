"""Every runtime module must export questions() and deploy()."""

from . import helpers


RUNTIMES_DIR = helpers.SRC_DIR / "runtimes"


def _runtime_modules():
    for file in sorted(RUNTIMES_DIR.rglob("*.py")):
        if file.name == "__init__.py":
            continue
        name = file.relative_to(RUNTIMES_DIR).parent.name
        yield name, file


def test_runtimes_have_questions_and_deploy():
    for name, path in _runtime_modules():
        ns = helpers.exec_module(path)
        assert callable(ns.get("questions")), f"{name}: missing questions()"
        assert callable(ns.get("deploy")), f"{name}: missing deploy()"


def test_laravel_uses_host_data():
    content = helpers.read(RUNTIMES_DIR / "laravel/laravel.py")
    helpers.assert_contains(content, "host.data", "laravel must use host.data")
