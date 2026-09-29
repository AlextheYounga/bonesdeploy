import re
from shlex import quote

from pyinfra.context import ctx_host
from pyinfra.facts.server import Arch, LinuxDistribution
from pyinfra.operations import server

PROFILE_CHECK_ATTEMPTS = 20
PROFILE_CHECK_INTERVAL_SECONDS = "0.1"
SUPPORTED_HOST_MESSAGE = "production hosts must be Debian 12+ or Ubuntu 24.04+ on x86_64"
MINIMUM_DEBIAN_VERSION = (12,)
MINIMUM_UBUNTU_VERSION = (24, 4)
_VERSION = re.compile(r"^\d+(?:\.\d+)*$")


def validate_supported_host() -> None:
    host = ctx_host.get()
    validate_supported_host_facts(host.get_fact(LinuxDistribution), host.get_fact(Arch))


def validate_supported_host_facts(distribution: object, architecture: object) -> None:
    release = distribution.get("release_meta") if isinstance(distribution, dict) else None
    if not isinstance(release, dict) or architecture != "x86_64":
        raise ValueError(SUPPORTED_HOST_MESSAGE)

    distribution_id = release.get("ID")
    version = _parse_version(release.get("VERSION_ID"))
    if distribution_id == "debian" and version is not None and version >= MINIMUM_DEBIAN_VERSION:
        return
    if distribution_id == "ubuntu" and version is not None and version >= MINIMUM_UBUNTU_VERSION:
        return
    raise ValueError(SUPPORTED_HOST_MESSAGE)


def _parse_version(value: object) -> tuple[int, ...] | None:
    if not isinstance(value, str) or not _VERSION.fullmatch(value):
        return None
    return tuple(int(part) for part in value.split("."))


def run_as_runtime_user(ctx, name, command):
    user = ctx.runtime.runtime_user
    q_user = quote(user)
    home = f"$(getent passwd {q_user} | cut -d: -f6)"
    wrapped = f"HOME={home} XDG_CONFIG_HOME={home}/.config {command}"
    server.shell(name=name, commands=[wrapped], _sudo=True, _sudo_user=user, _chdir=ctx.paths.current)


def verify_profile_attached(service_name, profile_name, *, name=None):
    q_service = quote(service_name)
    q_profile = quote(profile_name)
    command = (
        f'attempt=0; while [ "$attempt" -lt {PROFILE_CHECK_ATTEMPTS} ]; do '
        f"if systemctl is-active --quiet {q_service}; then "
        f"pid=$(systemctl show -p MainPID --value {q_service}); "
        f'if [ "$pid" != "0" ] && [ -n "$pid" ] && '
        f"grep -qF -- {q_profile} /proc/$pid/attr/current; then exit 0; fi; "
        f"fi; attempt=$((attempt + 1)); sleep {PROFILE_CHECK_INTERVAL_SECONDS}; done; "
        f"systemctl status {q_service} --no-pager --full >&2; "
        f"journalctl -u {q_service} -n 50 --no-pager >&2; false"
    )
    server.shell(name=name or f"Verify {service_name} attached to {profile_name}", commands=[command], _sudo=True)
