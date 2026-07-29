import importlib
from bonesinfra.frameworks import list_frameworks

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
        framework = getattr(mod, f"{name.upper()}_FRAMEWORK", None)
        assert framework is not None, f"{name}: missing {name.upper()}_FRAMEWORK"
        assert callable(getattr(framework, "deploy", None)), f"{name}: framework.deploy() not callable"


def test_framework_registry_is_explicit():
    assert list_frameworks() == sorted(FRAMEWORKS_MODULES)


def test_next_declares_uses_tcp():
    mod = importlib.import_module("bonesinfra.frameworks.next")
    assert mod.NEXT_FRAMEWORK.uses_tcp is True


def test_nuxt_does_not_declare_uses_tcp():
    mod = importlib.import_module("bonesinfra.frameworks.nuxt")
    assert mod.NUXT_FRAMEWORK.uses_tcp is False
