from shlex import quote

from pyinfra.context import ctx_host
from pyinfra.facts.server import Users
from pyinfra.operations import server


def _ensure_group_membership(user, group):
    q_user = quote(user)
    q_group = quote(group)
    server.shell(
        name=f"Ensure {user} is a member of {group}",
        commands=[f"id -nG {q_user} | tr ' ' '\\n' | grep -Fxq {q_group} || gpasswd -a {q_user} {q_group}"],
        _sudo=True,
    )


def ensure_users_and_groups(ctx):
    host = ctx_host.get()
    _ensure_site_identities(ctx, host)


def _ensure_site_identities(ctx, host):
    server.group(
        name="Ensure runtime group exists",
        group=ctx.runtime.runtime_group,
        _sudo=True,
    )

    existing_user = host.get_fact(Users).get(ctx.runtime.runtime_user)

    if existing_user is None:
        server.user(
            name="Ensure runtime user exists with groups",
            user=ctx.runtime.runtime_user,
            system=True,
            home="/nonexistent",
            shell="/usr/sbin/nologin",
            create_home=False,
            groups=[ctx.runtime.runtime_group],
            _sudo=True,
        )
    elif (
        ctx.runtime.runtime_group != existing_user["group"] and ctx.runtime.runtime_group not in existing_user["groups"]
    ):
        _ensure_group_membership(ctx.runtime.runtime_user, ctx.runtime.runtime_group)
