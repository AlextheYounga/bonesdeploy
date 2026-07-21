from shlex import quote

from pyinfra.operations import server


def run_as_runtime_user(ctx, name, command):
    """Validate a runtime command as the runtime user, never as root.

    Root provisioning must not also be the thing that proves the service can
    start: a root-owned validation artifact (e.g. a log file) would later be
    unwritable by the service. Runs the command with HOME/XDG_CONFIG_HOME set
    so user-scoped config resolution matches the real service environment.
    """
    user = ctx.runtime.runtime_user
    q_user = quote(user)
    home = f"$(getent passwd {q_user} | cut -d: -f6)"
    wrapped = f"HOME={home} XDG_CONFIG_HOME={home}/.config {command}"
    server.shell(
        name=name,
        commands=[wrapped],
        _sudo=True,
        _sudo_user=user,
    )


def verify_profile_attached(service_name, profile_name, *, name=None):
    """Verify the service's main PID is confined to the expected AppArmor profile.

    A loaded-but-unattached profile is a silent isolation failure: the service
    runs unrestricted. Reads /proc/<MainPID>/attr/current and fails if the
    profile name is not present. Deliberately run as root (needs /proc access
    to other users' attr and systemctl show).
    """
    q_service = quote(service_name)
    q_profile = quote(profile_name)
    cmd = (
        f"pid=$(systemctl show -p MainPID --value {q_service}); "
        f'[ "$pid" != "0" ] && [ -n "$pid" ] && '
        f"grep -qF -- {q_profile} /proc/$pid/attr/current || "
        f"{{ systemctl status {q_service} --no-pager --full >&2; "
        f"journalctl -u {q_service} -n 50 --no-pager >&2; false; }}"
    )
    server.shell(
        name=name or f"Verify {service_name} attached to {profile_name}",
        commands=[cmd],
        _sudo=True,
    )
