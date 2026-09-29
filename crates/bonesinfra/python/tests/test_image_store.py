"""Shared Podman image store provisioning checks."""

import tomllib

from jinja2 import Template

from bonesinfra.services.linux.image_store import BASE_IMAGE

from . import helpers

IMAGE_STORE_TEMPLATE = helpers.SRC_DIR / "bonesinfra/assets/podman/image-store-storage.conf.j2"
BUILD_USER_TEMPLATE = helpers.SRC_DIR / "bonesinfra/assets/podman/build-user-storage.conf.j2"
PINNED_BASE_IMAGE = (
    "docker.io/library/buildpack-deps:bookworm@sha256:5ac8377b7884040464fbf9390409073c6ac62ac8e6563e886fc560f9ea13ec5d"
)


def test_base_image_is_pinned_to_the_shared_builder_reference():
    assert BASE_IMAGE == PINNED_BASE_IMAGE


def test_image_store_template_overlay_section():
    c = helpers.read(IMAGE_STORE_TEMPLATE)
    parsed = tomllib.loads(c)
    assert parsed["storage"]["driver"] == "overlay"
    overlay = parsed["storage"]["options"]["overlay"]
    assert overlay["force_mask"] == "shared"
    assert overlay["mount_program"] == "/usr/bin/fuse-overlayfs"


def test_build_user_template_renders():
    c = helpers.read(BUILD_USER_TEMPLATE)
    rendered = Template(c).render(additional_image_store="/var/lib/bonesdeploy/image-store")
    helpers.assert_contains(rendered, "/var/lib/bonesdeploy/image-store")
