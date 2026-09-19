"""Typed provisioning request parsing at the process boundary."""

import pytest

from bonesinfra.config.context import DeployContext, ServerContext, template_data
from bonesinfra.config.request import parse_request

from .helpers import make_server_request, make_site_request


def test_reads_typed_site_request():
    ctx = DeployContext.from_request(make_site_request())

    assert ctx.app.project_name == "lawsnipe"
    assert ctx.app.repo_path == "/home/git/lawsnipe.git"
    assert ctx.app.project_root == "/srv/sites/lawsnipe"
    assert ctx.server.host == "example.com"
    assert ctx.server.port == "2222"
    assert ctx.paths.repo == "/home/git/lawsnipe.git"
    assert ctx.paths.project_root == "/srv/sites/lawsnipe"
    assert ctx.paths.current_web_root == "/srv/sites/lawsnipe/current/dist"
    assert ctx.app.deploy.branch == "main"
    assert ctx.app.dns.ssl_enabled is True
    assert ctx.runtime.runtime_user == "lawsnipe"
    assert ctx.runtime.runtime_group == "lawsnipe"


def test_template_data_contains_runtime_values():
    td = template_data(DeployContext.from_request(make_site_request()))
    assert td["runtime_user"] == "lawsnipe"
    assert td["runtime_group"] == "lawsnipe"
    assert td["runtime_backend"] == "native"


def test_extras_are_forwarded_with_json_types():
    request = make_site_request(extras={"php_version": True, "is_static": False, "custom": "value"})
    ctx = DeployContext.from_request(request)

    assert ctx.runtime.data == {"php_version": True, "is_static": False, "custom": "value"}
    assert template_data(ctx)["is_static"] is False


def test_ssl_enabled_accepts_boolean_string_for_request_compatibility():
    ctx = DeployContext.from_request(make_site_request(ssl_enabled="false"))
    assert ctx.app.dns.ssl_enabled is False


def test_docker_runtime_backend_is_preserved():
    ctx = DeployContext.from_request(make_site_request(backend="docker", compose_port=8080, compose_wait_timeout=60))
    assert ctx.runtime.backend == "docker"
    assert "backend" not in ctx.runtime.data
    assert ctx.runtime.compose_port == 8080
    assert ctx.runtime.compose_wait_timeout == 60


def test_unknown_runtime_backend_is_rejected():
    with pytest.raises(ValueError, match="RUNTIME_BACKEND"):
        DeployContext.from_request(make_site_request(backend="compose"))


def test_docker_with_managed_ingress_requires_compose_port():
    with pytest.raises(ValueError, match="compose_port is required"):
        DeployContext.from_request(make_site_request(backend="docker"))


def test_missing_site_fields_use_defaults():
    request = {
        "server": make_server_request()["server"],
        "site": {
            "project_name": "lawsnipe",
            "backup": {"schedule": "0 0 * * *", "retention_days": 30, "passphrase": ""},
        },
    }
    ctx = DeployContext.from_request(request)
    assert ctx.app.deploy.branch == "main"
    assert ctx.app.repo_path == "/home/git/lawsnipe.git"
    assert ctx.runtime.web_root == "public"
    assert ctx.runtime.runtime_user == "lawsnipe"
    assert ctx.backup.configured is False


def test_empty_template_uses_custom_framework():
    ctx = DeployContext.from_request(make_site_request(template=""))

    assert ctx.template == "custom"


@pytest.mark.parametrize("compose_port", [0, 65536, True, "8080"])
def test_invalid_compose_port_is_rejected(compose_port):
    with pytest.raises(ValueError, match="compose_port"):
        DeployContext.from_request(make_site_request(compose_port=compose_port))


@pytest.mark.parametrize("compose_wait_timeout", [0, 3601, True, "120"])
def test_invalid_compose_wait_timeout_is_rejected(compose_wait_timeout):
    with pytest.raises(ValueError, match="compose_wait_timeout"):
        DeployContext.from_request(make_site_request(compose_wait_timeout=compose_wait_timeout))


def test_unknown_request_fields_are_rejected():
    request = make_site_request()
    request["unexpected"] = "value"
    with pytest.raises(ValueError, match="unknown request field 'unexpected'"):
        DeployContext.from_request(request)


def test_unknown_site_fields_are_rejected():
    with pytest.raises(ValueError, match="unknown site field 'unexpected'"):
        DeployContext.from_request(make_site_request(unexpected="value"))


def test_invalid_extras_are_rejected():
    with pytest.raises(ValueError, match="must be a scalar"):
        DeployContext.from_request(make_site_request(extras={"nested": {"value": 1}}))


@pytest.mark.parametrize("project_name", ["", "Demo", "demo_name", "network", "demo;rm"])
def test_project_identity_matches_remote_validation(project_name):
    with pytest.raises(ValueError, match="Invalid project name"):
        DeployContext.from_request(make_site_request(project_name=project_name))


def test_server_context_uses_connection_values():
    ctx = ServerContext.from_request(make_server_request())
    assert (ctx.host, ctx.ssh_user, ctx.port) == ("example.com", "root", "2222")


def test_server_request_rejects_unknown_fields():
    with pytest.raises(ValueError, match="unknown server field 'invalid'"):
        parse_request({"server": {"host": "example.com", "invalid": True}}, server_only=True)


def test_service_request_fields_are_rejected():
    request = make_site_request()
    request["services"] = {}
    with pytest.raises(ValueError, match="unknown request field 'services'"):
        DeployContext.from_request(request)
