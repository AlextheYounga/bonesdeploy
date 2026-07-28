import sys

from bonesinfra.frameworks import laravel
from bonesinfra.frameworks.django import django
from bonesinfra.frameworks.next import next as next_runtime
from bonesinfra.frameworks.nuxt import nuxt
from bonesinfra.frameworks.rails import rails
from bonesinfra.frameworks.sveltekit import svelte
from bonesinfra.frameworks.vue import vue

FRAMEWORKS = {
    "laravel": laravel,
    "django": django,
    "next": next_runtime,
    "nuxt": nuxt,
    "rails": rails,
    "sveltekit": svelte,
    "vue": vue,
}


def list_frameworks():
    return sorted(FRAMEWORKS.keys())


def get_framework(name):
    module = FRAMEWORKS.get(name)
    if module is None:
        print(f"Unknown runtime: {name}. Available: {', '.join(list_frameworks())}", file=sys.stderr)
        sys.exit(1)
    return module
