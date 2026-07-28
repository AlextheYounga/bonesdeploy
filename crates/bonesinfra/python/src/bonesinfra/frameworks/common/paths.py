from pyinfra.operations import files

from bonesinfra.domain.paths import RUNTIME_SOCKET_PARENT


def ensure_runtime_dirs(ctx):
    """Create /run/<project> as the runtime user before validation or service start.

    RuntimeDirectory= in the systemd unit also does this at start time, but
    validation runs before start, so we provision it here too.
    """
    # 0711: system nginx (www-data) must traverse /run/<project>/ to reach
    # the per-site nginx socket. 0750 would cause 502 on every public request.
    project = ctx.app.project_name
    files.directory(
        name="Ensure runtime socket directory exists",
        path=f"{RUNTIME_SOCKET_PARENT}/{project}",
        user=ctx.runtime.runtime_user,
        group=ctx.runtime.runtime_group,
        mode="0711",
        _sudo=True,
    )
