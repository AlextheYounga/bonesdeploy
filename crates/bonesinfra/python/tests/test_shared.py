from bonesinfra.frameworks.django.runtime import SHARED_DIRECTORIES as DJANGO_SHARED_DIRECTORIES
from bonesinfra.frameworks.laravel.runtime import SHARED_DIRECTORIES as LARAVEL_SHARED_DIRECTORIES
from bonesinfra.frameworks.next.runtime import SHARED_DIRECTORIES as NEXT_SHARED_DIRECTORIES
from bonesinfra.frameworks.nuxt.runtime import SHARED_DIRECTORIES as NUXT_SHARED_DIRECTORIES
from bonesinfra.frameworks.rails.runtime import SHARED_DIRECTORIES as RAILS_SHARED_DIRECTORIES
from bonesinfra.frameworks.sveltekit.runtime import SHARED_DIRECTORIES as SVELTEKIT_SHARED_DIRECTORIES
from bonesinfra.frameworks.vue.runtime import SHARED_DIRECTORIES as VUE_SHARED_DIRECTORIES
from bonesinfra.services.linux import shared


def test_framework_shared_directories_are_directories_only():
    declarations = (
        DJANGO_SHARED_DIRECTORIES,
        LARAVEL_SHARED_DIRECTORIES,
        NEXT_SHARED_DIRECTORIES,
        NUXT_SHARED_DIRECTORIES,
        RAILS_SHARED_DIRECTORIES,
        SVELTEKIT_SHARED_DIRECTORIES,
        VUE_SHARED_DIRECTORIES,
    )

    assert all(
        path and not path.endswith("/") and "." not in path.rsplit("/", 1)[-1]
        for paths in declarations
        for path in paths
    )


def test_laravel_declares_all_shared_runtime_directories():
    assert LARAVEL_SHARED_DIRECTORIES == ("storage", "storage/framework/views", "cache", "uploads")


def test_ensure_directories_creates_only_declared_directories(monkeypatch):
    calls = []

    class Runtime:
        runtime_user = "atlas"
        runtime_group = "atlas"

    class Context:
        runtime = Runtime()

    monkeypatch.setattr(shared, "mkdir", lambda **kwargs: calls.append(kwargs))

    shared.ensure_directories(Context(), {"shared": "/srv/sites/atlas/shared"}, ("storage", "uploads"))

    assert [call["path"] for call in calls] == [
        "/srv/sites/atlas/shared/storage",
        "/srv/sites/atlas/shared/uploads",
    ]
    assert all(call["mode"] == "0770" for call in calls)
