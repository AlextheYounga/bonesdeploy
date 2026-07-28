import importlib
from types import SimpleNamespace

from bonesinfra.frameworks import list_frameworks
from bonesinfra.frameworks.laravel import php_fpm

FRAMEWORKS_MODULES = {
    "laravel": "bonesinfra.frameworks.laravel",
    "django": "bonesinfra.frameworks.django",
    "next": "bonesinfra.frameworks.next",
    "nuxt": "bonesinfra.frameworks.nuxt",
    "rails": "bonesinfra.frameworks.rails",
    "sveltekit": "bonesinfra.frameworks.sveltekit",
    "vue": "bonesinfra.frameworks.vue",
}


def test_frameworks_expose_framework_instance():
    for name, module_path in FRAMEWORKS_MODULES.items():
        mod = importlib.import_module(module_path)
        framework = getattr(mod, "FRAMEWORK", None)
        assert framework is not None, f"{name}: missing FRAMEWORK"
        assert callable(getattr(framework, "deploy", None)), f"{name}: FRAMEWORK.deploy() not callable"


def test_framework_registry_is_explicit():
    assert list_frameworks() == sorted(FRAMEWORKS_MODULES)


def test_laravel_php_fpm_cleans_orphaned_project_pools(monkeypatch):
    calls = []

    def fake_script_template(**kwargs):
        calls.append(kwargs)

    monkeypatch.setattr(php_fpm.server, "script_template", fake_script_template)
    ctx = SimpleNamespace(app=SimpleNamespace(project_name="demo"))

    php_fpm.cleanup_orphaned_pools(ctx, "8.5")

    assert len(calls) == 1
    assert calls[0]["project"] == "demo"
    assert "pool.d" in calls[0]["current_pool"]


def test_next_declares_uses_tcp():
    mod = importlib.import_module("bonesinfra.frameworks.next")
    assert mod.FRAMEWORK.uses_tcp is True


def test_nuxt_does_not_declare_uses_tcp():
    mod = importlib.import_module("bonesinfra.frameworks.nuxt")
    assert mod.FRAMEWORK.uses_tcp is False
