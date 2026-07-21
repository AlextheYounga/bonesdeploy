import sys

from bonesinfra.runtimes import laravel
from bonesinfra.runtimes.django import django
from bonesinfra.runtimes.next import next as next_runtime
from bonesinfra.runtimes.nuxt import nuxt
from bonesinfra.runtimes.rails import rails
from bonesinfra.runtimes.sveltekit import svelte
from bonesinfra.runtimes.vue import vue

RUNTIMES = {
    "laravel": laravel,
    "django": django,
    "next": next_runtime,
    "nuxt": nuxt,
    "rails": rails,
    "sveltekit": svelte,
    "vue": vue,
}


def list_runtimes():
    return sorted(RUNTIMES.keys())


def get_runtime(name):
    module = RUNTIMES.get(name)
    if module is None:
        print(f"Unknown runtime: {name}. Available: {', '.join(list_runtimes())}", file=sys.stderr)
        sys.exit(1)
    return module
