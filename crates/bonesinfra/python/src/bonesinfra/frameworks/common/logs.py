from pyinfra.operations import files

BONESDEPLOY_LOG_ROOT = "/var/log/bonesdeploy"


def ensure(ctx):
    """Provision /var/log/bonesdeploy and /var/log/bonesdeploy/<project>.

    The root is root-owned; the per-project dir is owned by the runtime user
    so the service (and validation) can write logs without root.
    """
    files.directory(
        name="Ensure BonesDeploy log root exists",
        path=BONESDEPLOY_LOG_ROOT,
        user="root",
        group="root",
        mode="0755",
        _sudo=True,
    )

    files.directory(
        name="Ensure per-project log directory exists",
        path=f"{BONESDEPLOY_LOG_ROOT}/{ctx.app.project_name}",
        user=ctx.runtime.runtime_user,
        group=ctx.runtime.runtime_group,
        mode="0750",
        _sudo=True,
    )
