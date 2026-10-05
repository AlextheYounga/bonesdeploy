from bonesinfra.pyinfra.operations import mkdir


def setup_project(ctx, paths):
    mkdir(
        name="Ensure control-plane site state directory exists",
        path=paths["site_root"],
        user="root",
        group="root",
        mode="0700",
    )

    mkdir(
        name="Ensure project root parent directory is traversable",
        path=paths["project_root_parent"],
        mode="0711",
    )

    mkdir(
        name="Ensure project root boundary exists",
        path=paths["project_root"],
        user="root",
        group="root",
        mode="0751",
    )

    mkdir(
        name="Ensure releases directory with setgid",
        path=paths["releases"],
        user="root",
        group=ctx.runtime.runtime_group,
        mode="2750",
    )

    mkdir(
        name="Ensure shared directory (owned by runtime user)",
        path=paths["shared"],
        user=ctx.runtime.runtime_user,
        group=ctx.runtime.runtime_group,
        mode="0750",
    )

    mkdir(
        name="Ensure root-controlled site runtime link directory exists",
        path=paths["site_runtime_dir"],
        user="root",
        group="root",
        mode="0755",
    )

    mkdir(
        name="Ensure placeholder release directory exists",
        path=paths["placeholder_web_root"],
        user="root",
        group=ctx.runtime.runtime_group,
        mode="0750",
    )
