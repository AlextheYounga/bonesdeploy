import importlib
from types import SimpleNamespace

from bonesinfra.frameworks import list_frameworks
from bonesinfra.frameworks.laravel import php_fpm

FRAMEWORKS_MODULES = {
    "laravel": "bonesinfra.frameworks.laravel",
    "django": "bonesinfra.frameworks.django.django",
    "next": "bonesinfra.frameworks.next.next",
    "nuxt": "bonesinfra.frameworks.nuxt.nuxt",
    "rails": "bonesinfra.frameworks.rails.rails",
    "sveltekit": "bonesinfra.frameworks.sveltekit.svelte",
    "vue": "bonesinfra.frameworks.vue.vue",
}


def test_runtimes_have_deploy():
    for name, module_path in FRAMEWORKS_MODULES.items():
        mod = importlib.import_module(module_path)
        assert callable(getattr(mod, "deploy", None)), f"{name}: missing deploy()"


def test_runtime_registry_is_explicit():
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
    mod = importlib.import_module("bonesinfra.frameworks.next.next")
    assert getattr(mod, "USES_TCP", False) is True


def test_nuxt_does_not_declare_uses_tcp():
    mod = importlib.import_module("bonesinfra.frameworks.nuxt.nuxt")
    assert not hasattr(mod, "USES_TCP")
