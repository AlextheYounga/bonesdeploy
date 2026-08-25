from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from bonesinfra.config.keys import (
    APP_KEYS,
    BRANCH,
    DOMAIN,
    EMAIL,
    HOST,
    PORT,
    PROJECT_NAME,
    RUNTIME_BACKEND,
    SERVICES,
    SSH_USER,
    SSL_ENABLED,
    SUPPORTED_DATABASE_SERVICES,
    WEB_ROOT,
)
from bonesinfra.config.paths import DEFAULT_PROJECT_ROOT_PARENT, DEFAULT_REPO_PARENT, DeploymentPaths

DEPLOY_USER = "git"

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

DEFAULT_SSH_USER = "root"
DEFAULT_SSH_PORT = "22"
DEFAULT_WEB_ROOT = "public"
MIN_QUOTED_VALUE_LENGTH = 2


@dataclass
class DeployContext:
    app: AppConfig
    runtime: RuntimeConfig
    services: ServicesConfig

    @classmethod
    def from_files(cls, config_path: str) -> DeployContext:
        values = read_dotenv(Path(config_path).read_text())
        project_name = values.get(PROJECT_NAME, "")
        _validate_project_name(project_name)

        app = AppConfig(
            project_name=project_name,
            repo_path=f"{DEFAULT_REPO_PARENT}/{project_name}.git",
            project_root=f"{DEFAULT_PROJECT_ROOT_PARENT}/{project_name}",
            server=ServerConfig(
                host=values.get(HOST, ""),
                ssh_user=values.get(SSH_USER, DEFAULT_SSH_USER),
                port=values.get(PORT, DEFAULT_SSH_PORT),
            ),
            dns=DnsConfig(
                domain=values.get(DOMAIN, ""),
                email=values.get(EMAIL, ""),
                ssl_enabled=values.get(SSL_ENABLED, "false").lower() == "true",
            ),
            deploy=DeployConfig(branch=values.get(BRANCH, "main")),
        )

        runtime = RuntimeConfig(
            backend=_runtime_backend(values.get(RUNTIME_BACKEND, "native")),
            web_root=values.get(WEB_ROOT) or DEFAULT_WEB_ROOT,
            runtime_user=project_name,
            runtime_group=project_name,
            data={key: value for key, value in values.items() if key not in APP_KEYS and key != "PREVIEW_DOMAIN"},
        )

        services_value = values.get(SERVICES, "")
        services = ServicesConfig(services=_database_services(services_value.split(",") if services_value else []))
        return cls(app=app, runtime=runtime, services=services)

    @property
    def paths(self) -> DeploymentPaths:
        try:
            return self._paths
        except AttributeError:
            pass
        self._paths = DeploymentPaths.new(
            self.app.project_name,
            self.app.repo_path,
            self.app.project_root,
            self.runtime.web_root,
        )
        return self._paths

    @property
    def paths_dict(self) -> dict[str, Any]:
        return self.paths.__dict__


def template_data(ctx: DeployContext, *, paths: dict[str, Any] | None = None, **extra: Any) -> dict[str, Any]:
    """Build flat template context from DeployContext for Jinja2 template rendering."""
    if paths is None:
        paths = ctx.paths_dict

    data: dict[str, Any] = {
        "project_name": ctx.app.project_name,
        "project_root": paths["project_root"],
        "web_root": ctx.runtime.web_root,
        "repo_path": paths["repo"],
        "branch": ctx.app.deploy.branch,
        "deploy_user": DEPLOY_USER,
        "runtime_user": ctx.runtime.runtime_user,
        "runtime_group": ctx.runtime.runtime_group,
        "runtime_backend": ctx.runtime.backend,
        "project_root_parent": paths["project_root_parent"],
        "ssh_port": int(ctx.app.server.port),
        "paths": paths,
        "ssl_domain": ctx.app.dns.domain,
        "ssl_email": ctx.app.dns.email,
    }

    for key, value in ctx.runtime.data.items():
        if key not in data:
            data[key] = value

    data.update(extra)
    return data


@dataclass
class AppConfig:
    project_name: str
    repo_path: str
    project_root: str
    server: ServerConfig
    dns: DnsConfig
    deploy: DeployConfig


@dataclass
class ServerConfig:
    ssh_user: str
    host: str
    port: str


@dataclass
class DnsConfig:
    domain: str
    email: str
    ssl_enabled: bool


@dataclass
class DeployConfig:
    branch: str


@dataclass
class RuntimeConfig:
    backend: str
    web_root: str
    runtime_user: str
    runtime_group: str
    data: dict[str, Any] = field(default_factory=dict)


@dataclass(frozen=True)
class ServicesConfig:
    services: tuple[str, ...] = ()


def read_dotenv(content: str) -> dict[str, str]:
    values: dict[str, str] = {}
    for line_number, raw_line in enumerate(content.splitlines(), 1):
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        key, separator, value = line.partition("=")
        key = key.strip()
        if not separator or not _valid_env_name(key):
            raise ValueError(f"invalid .env entry on line {line_number}")
        if key in values:
            raise ValueError(f"duplicate .env key {key!r} on line {line_number}")
        values[key] = _strip_quotes(value.strip())
    return values


def _valid_env_name(name: str) -> bool:
    return (
        bool(name)
        and ((name[0].isascii() and name[0].isalpha()) or name[0] == "_")
        and all((character.isascii() and character.isalnum()) or character == "_" for character in name[1:])
    )


def _strip_quotes(value: str) -> str:
    if len(value) >= MIN_QUOTED_VALUE_LENGTH and ((value[0] == value[-1] == '"') or (value[0] == value[-1] == "'")):
        return value[1:-1]
    return value


def _database_services(value: Any) -> tuple[str, ...]:
    if not isinstance(value, list) or not all(isinstance(service, str) for service in value):
        raise TypeError("SERVICES must be a comma-separated list")
    services = tuple(value)
    unsupported = set(services) - SUPPORTED_DATABASE_SERVICES
    if unsupported:
        raise ValueError(f"unsupported database services: {', '.join(sorted(unsupported))}")
    if "mariadb" in services and "mysql" in services:
        raise ValueError("mariadb and mysql cannot be provisioned together")
    if len(set(services)) != len(services):
        raise ValueError("database services must not contain duplicates")
    return services


def _runtime_backend(value: Any) -> str:
    if not isinstance(value, str) or value not in {"native", "docker"}:
        raise ValueError("RUNTIME_BACKEND must be 'native' or 'docker'")
    return value


def _validate_project_name(value: str) -> None:
    valid_characters = all(char.isascii() and (char.islower() or char.isdigit() or char == "-") for char in value)
    if value and valid_characters and value not in _RESERVED_PROJECT_NAMES:
        return
    raise ValueError(f"Invalid project name: {value}")
