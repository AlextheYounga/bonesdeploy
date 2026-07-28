import sys

from bonesinfra.frameworks.django import FRAMEWORK as _django
from bonesinfra.frameworks.laravel import FRAMEWORK as _laravel
from bonesinfra.frameworks.next import FRAMEWORK as _next
from bonesinfra.frameworks.nuxt import FRAMEWORK as _nuxt
from bonesinfra.frameworks.rails import FRAMEWORK as _rails
from bonesinfra.frameworks.sveltekit import FRAMEWORK as _sveltekit
from bonesinfra.frameworks.vue import FRAMEWORK as _vue

FRAMEWORKS = {
    "django": _django,
    "laravel": _laravel,
    "next": _next,
    "nuxt": _nuxt,
    "rails": _rails,
    "sveltekit": _sveltekit,
    "vue": _vue,
}


def list_frameworks():
    return sorted(FRAMEWORKS.keys())


def get_framework(name):
    framework = FRAMEWORKS.get(name)
    if framework is None:
        print(f"Unknown framework: {name}. Available: {', '.join(list_frameworks())}", file=sys.stderr)
        sys.exit(1)
    return framework
