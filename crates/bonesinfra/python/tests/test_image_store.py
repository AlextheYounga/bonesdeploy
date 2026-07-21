"""Tests for shared Podman image store provisioning."""

import tomllib

from jinja2 import Template

from . import helpers

IMAGE_STORE_MODULE = helpers.SRC_DIR / "bonesinfra/deploys/setup/image_store.py"
IMAGE_STORE_TEMPLATE = helpers.SRC_DIR / "bonesinfra/assets/podman/image-store-storage.conf.j2"
BUILD_USER_TEMPLATE = helpers.SRC_DIR / "bonesinfra/assets/podman/build-user-storage.conf.j2"
PATHS_MODULE = helpers.SRC_DIR / "bonesinfra/domain/paths.py"
SETUP_PLAN = helpers.SRC_DIR / "bonesinfra/deploys/setup/plan.py"
SETUP_USERS = helpers.SRC_DIR / "bonesinfra/deploys/setup/users.py"


def test_image_store_constants():
    c = helpers.read(PATHS_MODULE)
    helpers.assert_contains(c, "IMAGE_STORE_GRAPH_ROOT")
    helpers.assert_contains(c, "IMAGE_STORE_RUN_ROOT")


def test_image_store_graph_root_path():
    c = helpers.read(PATHS_MODULE)
    helpers.assert_contains(c, 'IMAGE_STORE_GRAPH_ROOT = "/var/lib/bonesdeploy/image-store"')


def test_image_store_run_root_path():
    c = helpers.read(PATHS_MODULE)
    helpers.assert_contains(c, 'IMAGE_STORE_RUN_ROOT = "/run/bonesdeploy/image-store"')


def test_image_store_module_exists():
    c = helpers.read(IMAGE_STORE_MODULE)
    helpers.assert_contains(c, "def ensure_shared_store")
    helpers.assert_contains(c, "def seed_base_image")


def test_image_store_creates_graph_root():
    c = helpers.read(IMAGE_STORE_MODULE)
    helpers.assert_contains(c, "IMAGE_STORE_GRAPH_ROOT")
    helpers.assert_contains(c, 'mode="0755"')


def test_image_store_creates_run_root():
    c = helpers.read(IMAGE_STORE_MODULE)
    helpers.assert_contains(c, "IMAGE_STORE_RUN_ROOT")


def test_image_store_storage_conf_path():
    c = helpers.read(IMAGE_STORE_MODULE)
    helpers.assert_contains(c, 'IMAGE_STORE_STORAGE_CONF = "/etc/bonesdeploy/image-store-storage.conf"')


def test_image_store_renders_storage_conf():
    c = helpers.read(IMAGE_STORE_MODULE)
    helpers.assert_contains(c, "Install shared image store storage configuration")
    helpers.assert_contains(c, "image-store-storage.conf.j2")


def test_image_store_template_overlay_section():
    c = helpers.read(IMAGE_STORE_TEMPLATE)
    parsed = tomllib.loads(c)
    assert parsed["storage"]["driver"] == "overlay"
    overlay = parsed["storage"]["options"]["overlay"]
    assert overlay["force_mask"] == "shared"
    assert overlay["mount_program"] == "/usr/bin/fuse-overlayfs"


def test_image_store_template_graphroot():
    c = helpers.read(IMAGE_STORE_TEMPLATE)
    helpers.assert_contains(c, 'graphroot = "/var/lib/bonesdeploy/image-store"')


def test_image_store_template_runroot():
    c = helpers.read(IMAGE_STORE_TEMPLATE)
    helpers.assert_contains(c, 'runroot = "/run/bonesdeploy/image-store"')


def test_image_store_pull_uses_dedicated_conf():
    c = helpers.read(IMAGE_STORE_MODULE)
    helpers.assert_contains(c, "CONTAINERS_STORAGE_CONF")
    helpers.assert_contains(c, "podman pull")


def test_image_store_pull_base_image():
    c = helpers.read(IMAGE_STORE_MODULE)
    helpers.assert_contains(c, "docker.io/library/buildpack-deps:bookworm")


def test_image_store_verify_image_exists():
    c = helpers.read(IMAGE_STORE_MODULE)
    helpers.assert_contains(c, "podman image exists")


def test_image_store_prune_dangling():
    c = helpers.read(IMAGE_STORE_MODULE)
    helpers.assert_contains(c, "podman image prune --force")


def test_image_store_normalizes_permissions():
    c = helpers.read(IMAGE_STORE_MODULE)
    helpers.assert_contains(c, "chmod -R a+rX")


def test_image_store_normalization_after_prune():
    c = helpers.read(IMAGE_STORE_MODULE)
    helpers.assert_ordering(
        c,
        "podman image prune",
        "chmod -R a+rX",
    )


def test_image_store_normalization_before_build_user_verification():
    c = helpers.read(SETUP_PLAN)
    helpers.assert_ordering(
        c,
        "image_store.seed_base_image",
        "users.ensure_users_and_groups",
    )


def test_build_user_template_additional_image_stores():
    c = helpers.read(BUILD_USER_TEMPLATE)
    helpers.assert_contains(c, "additionalimagestores")
    helpers.assert_contains(c, "{{ additional_image_store }}")


def test_build_user_template_fuse_overlayfs():
    c = helpers.read(BUILD_USER_TEMPLATE)
    helpers.assert_contains(c, 'mount_program = "/usr/bin/fuse-overlayfs"')


def test_build_user_template_driver():
    c = helpers.read(BUILD_USER_TEMPLATE)
    helpers.assert_contains(c, 'driver = "overlay"')


def test_build_user_template_renders():
    c = helpers.read(BUILD_USER_TEMPLATE)
    rendered = Template(c).render(additional_image_store="/var/lib/bonesdeploy/image-store")
    helpers.assert_contains(rendered, "/var/lib/bonesdeploy/image-store")


def test_users_configure_build_user_storage():
    c = helpers.read(SETUP_USERS)
    helpers.assert_contains(c, "def configure_build_user_storage")


def test_users_configure_build_user_storage_creates_config_dir():
    c = helpers.read(SETUP_USERS)
    helpers.assert_contains(c, "Ensure containers config directory")
    helpers.assert_contains(c, "config_parent")
    helpers.assert_contains(c, "config_dir")


def test_users_configure_build_user_storage_renders_template():
    c = helpers.read(SETUP_USERS)
    helpers.assert_contains(c, "build-user-storage.conf.j2")


def test_users_configure_build_user_storage_sets_permissions():
    c = helpers.read(SETUP_USERS)
    storage_block = c.split("def configure_build_user_storage")[1].split("\n\ndef ")[0]
    helpers.assert_contains(storage_block, 'mode="0700"')


def test_users_configure_build_user_storage_owns_dot_config():
    c = helpers.read(SETUP_USERS)
    storage_block = c.split("def configure_build_user_storage")[1].split("\n\ndef ")[0]
    helpers.assert_contains(storage_block, "Ensure .config directory for")
    helpers.assert_contains(storage_block, '.config"')


def test_users_configure_build_user_storage_dot_config_before_containers():
    c = helpers.read(SETUP_USERS)
    helpers.assert_ordering(
        c,
        "Ensure .config directory",
        "Ensure containers config directory",
    )


def test_users_verify_base_image_exists():
    c = helpers.read(SETUP_USERS)
    helpers.assert_contains(c, "Verify shared image store")
    helpers.assert_contains(c, "podman image exists {quote(BASE_IMAGE)}")


def test_users_uses_base_image_constant():
    c = helpers.read(SETUP_USERS)
    helpers.assert_contains(c, "from bonesinfra.deploys.setup.image_store import BASE_IMAGE")


def test_users_configure_storage_before_verification():
    c = helpers.read(SETUP_USERS)
    helpers.assert_ordering(
        c,
        "configure_build_user_storage",
        "Verify rootless Podman",
    )


def test_setup_plan_calls_ensure_shared_store():
    c = helpers.read(SETUP_PLAN)
    helpers.assert_contains(c, "image_store.ensure_shared_store")


def test_setup_plan_calls_seed_base_image():
    c = helpers.read(SETUP_PLAN)
    helpers.assert_contains(c, "image_store.seed_base_image")


def test_setup_plan_image_store_ordering():
    c = helpers.read(SETUP_PLAN)
    helpers.assert_ordering(
        c,
        "packages.install_system",
        "image_store.ensure_shared_store",
    )
    helpers.assert_ordering(
        c,
        "image_store.seed_base_image",
        "users.ensure_users_and_groups",
    )


def test_setup_plan_image_store_before_directories():
    c = helpers.read(SETUP_PLAN)
    helpers.assert_ordering(
        c,
        "image_store.seed_base_image",
        "directories.setup_repo_and_project",
    )


def test_paths_build_cache_name_constant():
    c = helpers.read(PATHS_MODULE)
    helpers.assert_contains(c, 'BUILD_CACHE_NAME = "cache"')


def test_users_configure_build_user_cache():
    c = helpers.read(SETUP_USERS)
    helpers.assert_contains(c, "def configure_build_user_cache")


def test_users_configure_build_user_cache_creates_dir():
    c = helpers.read(SETUP_USERS)
    cache_block = c.split("def configure_build_user_cache")[1].split("\n\ndef ")[0]
    helpers.assert_contains(cache_block, "Ensure persistent build cache")
    helpers.assert_contains(cache_block, "mkdir(")
    helpers.assert_contains(cache_block, 'mode="0700"')


def test_users_configure_build_user_cache_uses_helper():
    c = helpers.read(SETUP_USERS)
    cache_block = c.split("def configure_build_user_cache")[1].split("\n\ndef ")[0]
    helpers.assert_contains(cache_block, "build_cache_for")


def test_users_verify_build_cache_exists():
    c = helpers.read(SETUP_USERS)
    helpers.assert_contains(c, "Verify build cache")
    helpers.assert_contains(c, "test -d")
    helpers.assert_contains(c, "test -O")
    helpers.assert_contains(c, "build cache missing or unsafe")


def test_users_configure_cache_before_verification():
    c = helpers.read(SETUP_USERS)
    helpers.assert_ordering(
        c,
        "configure_build_user_cache",
        "Verify rootless Podman",
    )
    helpers.assert_ordering(
        c,
        "configure_build_user_cache",
        "Verify build cache",
    )
