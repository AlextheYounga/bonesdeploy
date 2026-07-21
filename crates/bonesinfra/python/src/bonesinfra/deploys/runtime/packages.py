from pyinfra.operations import apt


def install_apt(ctx):
    pkgs = ctx.runtime.data.get("runtime_apt_packages", [])
    if not pkgs:
        return
    apt.packages(
        name="Install runtime apt packages",
        packages=pkgs,
        present=True,
        update=True,
        cache_time=3600,
        _sudo=True,
    )
