"""Typed provisioning request parsing at the Python process boundary."""

import re
from collections.abc import Mapping
from typing import Any

# ValueError is part of the public boundary contract for malformed requests.
# ruff: noqa: TRY004
from bonesinfra.config.context import (
    AppConfig,
    BackupConfig,
    DeployContext,
    DnsConfig,
    RuntimeConfig,
    ServerContext,
)
from bonesinfra.config.paths import DEFAULT_PROJECT_ROOT_PARENT, DEFAULT_WEB_ROOT

_RESERVED_PROJECT_NAMES = {
    "basic",
    "default",
    "emergency",
    "final",
    "graphical",
    "halt",
    "initrd",
    "local-fs",
    "multi-user",
    "network",
    "network-online",
    "poweroff",
    "reboot",
    "remote-fs",
    "rescue",
    "shutdown",
    "sockets",
    "swap",
    "sysinit",
    "system-update",
    "timers",
    "umount",
}
_FRAMEWORKS = {"custom", "django", "laravel", "next", "nuxt", "rails", "sveltekit", "vue"}
_DOMAIN_LABEL = re.compile(r"^[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?$")
_EMAIL = re.compile(r"^[^@\s]+@[^@\s]+\.[^@\s]+$")
_CRON_FIELD = re.compile(r"^[0-9A-Za-z*/,\-]+$")
_CRON_FIELD_COUNT = 5
_MAX_CRON_FIELD_LENGTH = 64
_MAX_DOMAIN_LENGTH = 253
_MIN_DOMAIN_LABELS = 2
_MAX_EMAIL_LENGTH = 254
_MAX_PORT = 65535
_MAX_COMPOSE_WAIT_TIMEOUT = 3600


def reject_unknown(mapping: Mapping[str, Any], allowed: set[str], where: str) -> None:
    for key in mapping:
        if key not in allowed:
            raise ValueError(f"unknown {where} field '{key}'")


def parse_server_connection(server: Mapping[str, Any]) -> ServerContext:
    reject_unknown(server, {"host", "ssh_user", "port"}, "server")
    host = _string(server.get("host", ""), "server.host")
    ssh_user = _string(server.get("ssh_user", ""), "server.ssh_user") or "root"
    port = _string(server.get("port", "22"), "server.port")
    if not port.isdigit():
        raise ValueError("server.port must contain only digits")
    return ServerContext(host=host, ssh_user=ssh_user, port=port)


def parse_site(body: Mapping[str, Any]) -> DeployContext:  # noqa: C901
    reject_unknown(body, {"server", "site"}, "request")
    server = body.get("server")
    site = body.get("site")
    if not isinstance(server, Mapping):
        raise ValueError("server is required")
    if not isinstance(site, Mapping):
        raise ValueError("site is required")
    reject_unknown(
        site,
        {
            "project_name",
            "domain",
            "email",
            "template",
            "backend",
            "web_root",
            "node_version",
            "compose_port",
            "compose_wait_timeout",
            "backup",
            "extras",
        },
        "site",
    )
    name = _string(site.get("project_name", ""), "site.project_name")
    _validate_project_name(name)
    backend = site.get("backend", "native")
    if backend not in {"native", "docker"}:
        raise ValueError("RUNTIME_BACKEND must be 'native' or 'docker'")
    template = _string(site.get("template", "custom"), "site.template") or "custom"
    if template not in _FRAMEWORKS:
        raise ValueError(f"unknown framework infrastructure: {template}")
    extras = site.get("extras", {})
    if not isinstance(extras, Mapping):
        raise ValueError("site.extras must be an object")
    for key, value in extras.items():
        if isinstance(value, (list, dict)):
            raise ValueError(f"site.extras.{key} must be a scalar")
    backup = _parse_backup(site.get("backup"))
    domain = _string(site.get("domain", ""), "site.domain")
    email = _string(site.get("email", ""), "site.email")
    compose_port = _optional_port(site.get("compose_port"))
    if domain:
        validate_domain(domain)
    if email:
        validate_email(email)
    if backend == "docker" and domain and compose_port is None:
        raise ValueError("site.compose_port is required for Docker sites with managed ingress")
    return DeployContext(
        server=parse_server_connection(server),
        app=AppConfig(
            name,
            f"{DEFAULT_PROJECT_ROOT_PARENT}/{name}",
            DnsConfig(domain, email),
        ),
        runtime=RuntimeConfig(
            backend,
            _string(site.get("web_root", ""), "site.web_root") or DEFAULT_WEB_ROOT,
            name,
            name,
            compose_port,
            _compose_wait_timeout(site.get("compose_wait_timeout", 120)),
            dict(extras),
        ),
        backup=backup,
        template=template,
    )


def parse_request(body: Mapping[str, Any], *, server_only: bool = False) -> DeployContext | ServerContext:
    reject_unknown(body, {"server", "site"}, "request")
    if server_only:
        if "site" in body:
            raise ValueError("unknown request field 'site'")
        server = body.get("server")
        if not isinstance(server, Mapping):
            raise ValueError("server is required")
        return parse_server_connection(server)
    return parse_site(body)


def _optional_port(value: Any) -> int | None:
    if value is None:
        return None
    if not isinstance(value, int) or isinstance(value, bool) or not 1 <= value <= _MAX_PORT:
        raise ValueError("site.compose_port must be an integer from 1 through 65535")
    return value


def _compose_wait_timeout(value: Any) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or not 1 <= value <= _MAX_COMPOSE_WAIT_TIMEOUT:
        raise ValueError("site.compose_wait_timeout must be an integer from 1 through 3600")
    return value


def _parse_backup(value: Any) -> BackupConfig:
    """Validate the backup section before it reaches the cron file or Borg repository."""
    if not isinstance(value, Mapping):
        raise ValueError("site.backup is required and must be an object")
    reject_unknown(value, {"schedule", "retention_days", "passphrase"}, "site.backup")
    schedule = _string(value.get("schedule", ""), "site.backup.schedule")
    _validate_cron_schedule(schedule)
    retention_days = value.get("retention_days")
    if not isinstance(retention_days, int) or isinstance(retention_days, bool) or retention_days < 1:
        raise ValueError("site.backup.retention_days must be a positive integer")
    passphrase = _string(value.get("passphrase", ""), "site.backup.passphrase")
    return BackupConfig(schedule, retention_days, passphrase)


def _validate_cron_schedule(schedule: str) -> None:
    fields = schedule.split()
    if len(fields) != _CRON_FIELD_COUNT:
        raise ValueError("site.backup.schedule must be a five-field crontab expression")
    for field in fields:
        if len(field) > _MAX_CRON_FIELD_LENGTH or not _CRON_FIELD.fullmatch(field):
            raise ValueError(f"site.backup.schedule field contains unsupported characters: {field!r}")


def _validate_project_name(value: str) -> None:
    if (
        value
        and all(char.isascii() and (char.islower() or char.isdigit() or char == "-") for char in value)
        and value not in _RESERVED_PROJECT_NAMES
    ):
        return
    raise ValueError(f"Invalid project name: {value}")


def validate_domain(value: str) -> str:
    """Validate a DNS name before it reaches generated configuration or shell commands."""
    if len(value) > _MAX_DOMAIN_LENGTH or value.endswith("."):
        raise ValueError("site.domain must be a valid DNS name")
    labels = value.split(".")
    if len(labels) < _MIN_DOMAIN_LABELS or any(not _DOMAIN_LABEL.fullmatch(label) for label in labels):
        raise ValueError("site.domain must be a valid DNS name")
    return value


def validate_email(value: str) -> str:
    """Validate the basic email shape required by Certbot."""
    if len(value) > _MAX_EMAIL_LENGTH or not value.isascii() or not _EMAIL.fullmatch(value):
        raise ValueError("site.email must be a valid email address")
    return value


def _string(value: Any, name: str) -> str:
    if not isinstance(value, str):
        raise ValueError(f"{name} must be a string")
    return value
