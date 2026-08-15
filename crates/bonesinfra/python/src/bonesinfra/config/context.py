from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from bonesinfra.config.paths import DEFAULT_PROJECT_ROOT_PARENT, DEFAULT_REPO_PARENT, DeploymentPaths

DEPLOY_USER = "git"

DEFAULT_SSH_USER = "root"
DEFAULT_SSH_PORT = "22"
DEFAULT_WEB_ROOT = "public"


@dataclass
class DeployContext:
    app: AppConfig
    runtime: RuntimeConfig
    services: ServicesConfig

    @classmethod
    def from_files(cls, config_path: str) -> DeployContext:
        values = _dotenv(Path(config_path).read_text())
        project_name = values.get("PROJECT_NAME", "")

        app = AppConfig(
            project_name=project_name,
            repo_path=f"{DEFAULT_REPO_PARENT}/{project_name}.git",
            project_root=f"{DEFAULT_PROJECT_ROOT_PARENT}/{project_name}",
            server=ServerConfig(
                host=values.get("HOST", ""),
                ssh_user=values.get("SSH_USER", DEFAULT_SSH_USER),
                port=values.get("PORT", DEFAULT_SSH_PORT),
            ),
            dns=DnsConfig(
                domain=values.get("DOMAIN", ""),
                preview_domain=values.get("PREVIEW_DOMAIN", ""),
                email=values.get("EMAIL", ""),
                ssl_enabled=values.get("SSL_ENABLED", "false").lower() == "true",
            ),
            deploy=DeployConfig(branch=values.get("BRANCH", "main")),
        )

        runtime = RuntimeConfig(
            backend=_runtime_backend(values.get("RUNTIME_BACKEND", "native")),
            web_root=values.get("WEB_ROOT") or DEFAULT_WEB_ROOT,
            runtime_user=project_name,
            runtime_group=project_name,
            data={
                key: value
                for key, value in values.items()
                if key
                not in {
                    "PROJECT_NAME",
                    "HOST",
                    "SSH_USER",
                    "PORT",
                    "DOMAIN",
                    "PREVIEW_DOMAIN",
                    "EMAIL",
                    "SSL_ENABLED",
                    "BRANCH",
                    "RUNTIME_BACKEND",
                    "WEB_ROOT",
                    "SERVICES",
                    "TEMPLATE",
                }
            },
        )

        services_value = values.get("SERVICES", "")
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
        "preview_domain": ctx.app.dns.preview_domain,
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
    preview_domain: str
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


def _dotenv(content: str) -> dict[str, str]:
    values: dict[str, str] = {}
    for line_number, raw_line in enumerate(content.splitlines(), 1):
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        key, separator, value = line.partition("=")
        if not separator or not key.strip():
            raise ValueError(f"invalid .env entry on line {line_number}")
        values[key.strip()] = value.strip().strip('"')
    return values


def _database_services(value: Any) -> tuple[str, ...]:
    if not isinstance(value, list) or not all(isinstance(service, str) for service in value):
        raise TypeError("SERVICES must be a comma-separated list")
    supported = {"postgres", "mariadb", "mysql", "mongodb", "valkey", "redis"}
    services = tuple(value)
    unsupported = set(services) - supported
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
