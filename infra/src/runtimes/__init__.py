import importlib
import src.runtimes.laravel.laravel
import src.runtimes.laravel.laravel
import src.runtimes.django.django
import src.runtimes.next.next
import src.runtimes.rails.rails
import src.runtimes.sveltekit.svelte
import src.runtimes.vue.vue


_REGISTRY = {}

_MODULE_PATHS = {
    "laravel": "src.runtimes.laravel.laravel",
    "django": "src.runtimes.django.django",
    "next": "src.runtimes.next.next",
    "rails": "src.runtimes.rails.rails",
    "sveltekit": "src.runtimes.sveltekit.svelte",
    "vue": "src.runtimes.vue.vue",
}


def _discover():
    for name, module_path in _MODULE_PATHS.items():
        try:
            module = importlib.import_module(module_path)
            _REGISTRY[name] = module
        except ImportError:
            pass


_discover()


def list_runtimes():
    return sorted(_REGISTRY.keys())


def get_runtime(name):
    module = _REGISTRY.get(name)
    if module is None:
        raise KeyError(
            f"Unknown runtime: {name}. Available: {', '.join(list_runtimes())}"
        )
    return module
