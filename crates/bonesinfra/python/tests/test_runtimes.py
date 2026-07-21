import importlib
from types import SimpleNamespace

from bonesinfra.runtimes import list_runtimes
from bonesinfra.runtimes.laravel import php_fpm

RUNTIMES_MODULES = {
    "laravel": "bonesinfra.runtimes.laravel",
    "django": "bonesinfra.runtimes.django.django",
    "next": "bonesinfra.runtimes.next.next",
    "nuxt": "bonesinfra.runtimes.nuxt.nuxt",
    "rails": "bonesinfra.runtimes.rails.rails",
    "sveltekit": "bonesinfra.runtimes.sveltekit.svelte",
    "vue": "bonesinfra.runtimes.vue.vue",
}


def test_runtimes_have_deploy():
    for name, module_path in RUNTIMES_MODULES.items():
        mod = importlib.import_module(module_path)
        assert callable(getattr(mod, "deploy", None)), f"{name}: missing deploy()"


def test_runtime_registry_is_explicit():
    assert list_runtimes() == sorted(RUNTIMES_MODULES)


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
    mod = importlib.import_module("bonesinfra.runtimes.next.next")
    assert getattr(mod, "USES_TCP", False) is True


def test_nuxt_does_not_declare_uses_tcp():
    mod = importlib.import_module("bonesinfra.runtimes.nuxt.nuxt")
    assert not hasattr(mod, "USES_TCP")
