import sys

from bonesinfra.frameworks.django import DJANGO_FRAMEWORK
from bonesinfra.frameworks.laravel import LARAVEL_FRAMEWORK
from bonesinfra.frameworks.next import NEXT_FRAMEWORK
from bonesinfra.frameworks.nuxt import NUXT_FRAMEWORK
from bonesinfra.frameworks.rails import RAILS_FRAMEWORK
from bonesinfra.frameworks.sveltekit import SVELTEKIT_FRAMEWORK
from bonesinfra.frameworks.vue import VUE_FRAMEWORK

FRAMEWORKS = {
    "django": DJANGO_FRAMEWORK,
    "laravel": LARAVEL_FRAMEWORK,
    "next": NEXT_FRAMEWORK,
    "nuxt": NUXT_FRAMEWORK,
    "rails": RAILS_FRAMEWORK,
    "sveltekit": SVELTEKIT_FRAMEWORK,
    "vue": VUE_FRAMEWORK,
}


def list_frameworks():
    return sorted(FRAMEWORKS.keys())


def get_framework(name):
    framework = FRAMEWORKS.get(name)
    if framework is None:
        print(f"Unknown framework: {name}. Available: {', '.join(list_frameworks())}", file=sys.stderr)
        sys.exit(1)
    return framework
