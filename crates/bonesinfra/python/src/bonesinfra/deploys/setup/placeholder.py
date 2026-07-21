from pyinfra.operations import files

from bonesinfra.domain.context import template_data
from bonesinfra.domain.paths import ASSETS_DIR
from bonesinfra.infra.deploy_helpers import render


def seed(ctx, paths):
    render(
        "Seed placeholder index page",
        ASSETS_DIR / "nginx/index.html.j2",
        paths["placeholder_index"],
        user="root",
        group=ctx.runtime.runtime_group,
        mode="0640",
        **template_data(ctx, paths=paths),
    )

    files.link(
        name="Point current symlink at placeholder release",
        path=paths["current"],
        target=paths["placeholder_release"],
        force=True,
        _sudo=True,
    )
