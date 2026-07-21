import importlib
from types import SimpleNamespace

from bonesinfra.runtimes import list_runtimes
from bonesinfra.runtimes.laravel import php_fpm

from . import helpers

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


def test_common_php_fpm_pool_socket_path_is_distro_standard():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/common/php_fpm_pool.py")
    helpers.assert_contains(content, '"/run/php"')
    helpers.assert_contains(content, "fpm/pool.d/{project}.conf")
    helpers.assert_contains(content, "php-fpm{php_version} --test")
    helpers.assert_contains(content, "php{php_version}-fpm")


def test_common_php_fpm_pool_ensures_bonesdeploy_log_dir():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/common/php_fpm_pool.py")
    helpers.assert_contains(content, "logs.ensure(ctx)")


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


def test_laravel_nginx_uses_distro_php_socket_and_no_runtime_chown():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/laravel/nginx.py")
    helpers.assert_contains(content, "php_fpm_pool.socket_path")
    helpers.assert_not_contains(content, "runtime_socket_dir")
    helpers.assert_not_contains(content, "files.directory")


def test_common_validation_runs_as_runtime_user_not_root():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/common/validation.py")
    helpers.assert_contains(content, "_sudo_user=user")
    helpers.assert_contains(content, "def run_as_runtime_user(ctx, name, command):")


def test_common_logs_provisions_runtime_owned_dir():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/common/logs.py")
    helpers.assert_contains(content, "/var/log/bonesdeploy")
    helpers.assert_contains(content, "user=ctx.runtime.runtime_user")
    helpers.assert_contains(content, 'name="Ensure BonesDeploy log root exists"')


def test_template_runtime_load_fails_loudly_without_silent_swallow():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/deploys/runtime/template_runtime.py")
    helpers.assert_not_contains(content, "except (ImportError, KeyError)")
    helpers.assert_not_contains(content, "    pass")


def test_vue_is_static_only():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/vue/vue.py")
    helpers.assert_contains(content, "nginx.render_static")
    helpers.assert_not_contains(content, "render_app_service")


def test_next_uses_tcp_localhost():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/next/next.py")
    helpers.assert_contains(content, "HOSTNAME=127.0.0.1")
    helpers.assert_contains(content, "port=port")
    helpers.assert_contains(content, 'runtime_address_families="AF_UNIX AF_INET"')


def test_nuxt_uses_nitro_unix_socket():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/nuxt/nuxt.py")
    helpers.assert_contains(content, "NITRO_UNIX_SOCKET=")
    helpers.assert_contains(content, "socket_path=socket_path")


def test_nuxt_static_path_uses_render_static():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/nuxt/nuxt.py")
    helpers.assert_contains(content, "nginx.render_static")
    helpers.assert_contains(content, "STATIC_ROOT")


def test_nuxt_static_runtime_seeds_placeholder_output():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/nuxt/nuxt.py")
    helpers.assert_contains(content, 'STATIC_ROOT = ".output/public"')
    helpers.assert_contains(content, "paths['placeholder_release']")
    helpers.assert_contains(content, '"Seed Nuxt static placeholder index page"')
    helpers.assert_contains(content, 'f"{static_web_root}/index.html"')


def test_vue_static_uses_dist_root():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/vue/vue.py")
    helpers.assert_contains(content, "nginx.render_static")
    helpers.assert_contains(content, 'VUE_STATIC_ROOT = "dist"')


def test_sveltekit_uses_socket_path_env():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/sveltekit/svelte.py")
    helpers.assert_contains(content, "SOCKET_PATH=")
    helpers.assert_contains(content, "ORIGIN=")


def test_django_uses_gunicorn_unix_socket():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/django/django.py")
    helpers.assert_contains(content, "gunicorn")
    helpers.assert_contains(content, "--bind unix:")
    helpers.assert_contains(content, "wsgi_module")
    helpers.assert_not_contains(content, "python3-gunicorn")


def test_app_runtimes_use_per_service_socket_leaf():
    """Each app runtime must place its socket in its own leaf dir under
    /run/<project>/<runtime>/, not in the shared /run/<project>/."""
    for slug, rel in [
        ("gunicorn", "bonesinfra/runtimes/django/django.py"),
        ("puma", "bonesinfra/runtimes/rails/rails.py"),
        ("nuxt", "bonesinfra/runtimes/nuxt/nuxt.py"),
        ("sveltekit", "bonesinfra/runtimes/sveltekit/svelte.py"),
    ]:
        content = helpers.read(helpers.SRC_DIR / rel)
        helpers.assert_contains(
            content,
            f"runtime_socket_dir']}}/{slug}/{slug}.sock",
            msg=slug,
        )


def test_rails_uses_puma_unix_socket():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/rails/rails.py")
    helpers.assert_contains(content, "bundle exec puma")
    helpers.assert_contains(content, "-b unix://")


def test_next_static_path_uses_render_static():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/next/next.py")
    helpers.assert_contains(content, "nginx.render_static")
    helpers.assert_contains(content, 'STATIC_ROOT = "out"')


def test_next_static_runtime_seeds_placeholder_output():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/next/next.py")
    helpers.assert_contains(content, "paths['placeholder_release']")
    helpers.assert_contains(content, '"Seed Next static placeholder index page"')
    helpers.assert_contains(content, 'f"{static_web_root}/index.html"')


def test_next_app_server_seeds_placeholder_server():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/next/next.py")
    helpers.assert_contains(content, "placeholder-server.js.j2")
    helpers.assert_contains(content, ".next/standalone/server.js")
    helpers.assert_contains(content, "_seed_placeholder_server")


def test_next_validation_uses_absolute_current_path():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/next/next.py")
    helpers.assert_contains(content, "test -f {paths['current']}/.next/standalone/server.js")


def test_nuxt_app_server_seeds_placeholder_server():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/nuxt/nuxt.py")
    helpers.assert_contains(content, "placeholder-server.mjs.j2")
    helpers.assert_contains(content, ".output/server/index.mjs")
    helpers.assert_contains(content, "_seed_placeholder_server")


def test_sveltekit_seeds_placeholder_server():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/sveltekit/svelte.py")
    helpers.assert_contains(content, "placeholder-index.js.j2")
    helpers.assert_contains(content, "_seed_placeholder_server")


def test_sveltekit_seeds_blank_env():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/sveltekit/svelte.py")
    helpers.assert_contains(content, "touch {quote(paths['placeholder_release'])}/.env")


def test_sveltekit_validation_uses_absolute_current_path():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/sveltekit/svelte.py")
    helpers.assert_contains(content, "test -e {paths['current']}/build")


def test_django_seeds_placeholder_venv_and_wsgi():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/django/django.py")
    helpers.assert_contains(content, "placeholder-wsgi.py.j2")
    helpers.assert_contains(content, "python3 -m venv .venv")
    helpers.assert_contains(content, ".venv/bin/pip install gunicorn")
    helpers.assert_contains(content, "_seed_placeholder_server")


def test_rails_seeds_placeholder_gemfile_and_rack():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/rails/rails.py")
    helpers.assert_contains(content, "placeholder-Gemfile.j2")
    helpers.assert_contains(content, "placeholder-config.ru.j2")
    helpers.assert_contains(content, "bundle install")
    helpers.assert_contains(content, "_seed_placeholder_server")


def test_rails_validation_uses_absolute_current_path():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/rails/rails.py")
    helpers.assert_contains(content, "cd {quote(paths['current'])} && bundle exec puma --help")


def test_vue_seeds_dist_placeholder():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/vue/vue.py")
    helpers.assert_contains(content, 'VUE_STATIC_ROOT = "dist"')
    helpers.assert_contains(content, '"Seed Vue static placeholder index page"')
    helpers.assert_contains(content, "paths['placeholder_release']")


def test_next_declares_uses_tcp():
    mod = importlib.import_module("bonesinfra.runtimes.next.next")
    assert getattr(mod, "USES_TCP", False) is True


def test_nuxt_does_not_declare_uses_tcp():
    mod = importlib.import_module("bonesinfra.runtimes.nuxt.nuxt")
    assert not hasattr(mod, "USES_TCP")
