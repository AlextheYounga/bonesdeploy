from pyinfra.context import ctx_host
from pyinfra.facts.server import LinuxDistribution

from bonesinfra.config.context import DeployContext
from bonesinfra.services.linux import compose

from .helpers import SRC_DIR, make_site_request, read


def test_compose_provisioning_installs_plugin_registers_unit_and_renders_loopback_nginx(monkeypatch):
    ctx = DeployContext.from_request(make_site_request(backend="docker", compose_port=8080, compose_wait_timeout=45))
    calls = []

    monkeypatch.setattr(compose, "_remove_stale_managed_runtime", lambda _ctx: calls.append("cleanup"))
    monkeypatch.setattr(compose.apt, "packages", lambda **kwargs: calls.append(("packages", kwargs)))
    monkeypatch.setattr(compose.files, "directory", lambda **kwargs: calls.append(("directory", kwargs)))
    monkeypatch.setattr(compose.files, "download", lambda **kwargs: calls.append(("download", kwargs)))
    monkeypatch.setattr(compose.systemd, "service", lambda **kwargs: calls.append(("docker", kwargs)))
    monkeypatch.setattr(compose.server, "shell", lambda **kwargs: calls.append(("shell", kwargs)))
    monkeypatch.setattr(compose.site_systemd, "render_target", lambda *_args, **_kwargs: calls.append("target"))
    monkeypatch.setattr(compose.files, "template", lambda **kwargs: calls.append(("template", kwargs)))
    monkeypatch.setattr(
        compose.site_systemd, "register_service", lambda *_args, **kwargs: calls.append(("register", kwargs))
    )
    monkeypatch.setattr(compose.runtime, "setup", lambda *_args, **kwargs: calls.append(("setup", kwargs)))
    monkeypatch.setattr(compose.nginx_site, "render_proxy", lambda *_args, **kwargs: calls.append(("nginx", kwargs)))
    monkeypatch.setattr(compose.runtime, "reconcile_ingress", lambda *_args: calls.append("ingress"))
    monkeypatch.setattr(
        compose.site_systemd, "enable_and_start", lambda *_args, **kwargs: calls.append(("start", kwargs))
    )
    monkeypatch.setattr(compose.runtime, "start_services", lambda *_args: calls.append("nginx-start"))

    class FakeHost:
        def get_fact(self, fact):
            assert fact is LinuxDistribution
            return {"release_meta": {"ID": "ubuntu", "VERSION_CODENAME": "noble"}}

    with ctx_host.use(FakeHost()):
        compose.deploy(ctx)

    assert calls[0] == "cleanup"
    package_operations = [call[1] for call in calls if isinstance(call, tuple) and call[0] == "packages"]
    packages = [operation["packages"] for operation in package_operations]
    assert packages == [
        ["ca-certificates"],
        ["docker.io", "docker-compose", "podman-docker", "containerd", "runc"],
        ["docker-ce", "docker-ce-cli", "containerd.io", "docker-buildx-plugin", "docker-compose-plugin"],
    ]
    assert package_operations[1]["present"] is False
    source = next(
        call[1]
        for call in calls
        if isinstance(call, tuple) and call[0] == "template" and call[1]["name"] == "Install Docker package source"
    )
    assert source["distribution"] == "ubuntu"
    assert source["codename"] == "noble"
    assert any(
        call[0] == "shell" and call[1]["commands"] == ["docker compose version >/dev/null"]
        for call in calls
        if isinstance(call, tuple)
    )
    assert any(call[0] == "register" and call[1]["name"] == "compose" for call in calls if isinstance(call, tuple))
    assert ("setup", {"uses_tcp": True}) in calls
    assert any(call[0] == "nginx" and call[1]["port"] == 8080 for call in calls if isinstance(call, tuple))
    assert calls[-2] == ("start", {})
    assert calls[-1] == "nginx-start"


def test_compose_without_ingress_does_not_configure_nginx(monkeypatch):
    ctx = DeployContext.from_request(make_site_request(backend="docker", domain="", ssl_enabled=False))
    monkeypatch.setattr(compose, "_remove_stale_managed_runtime", lambda _ctx: None)
    monkeypatch.setattr(compose, "_install_docker", lambda: None)
    monkeypatch.setattr(compose.site_systemd, "render_target", lambda *_args, **_kwargs: None)
    monkeypatch.setattr(compose.files, "template", lambda **_kwargs: None)
    monkeypatch.setattr(compose.site_systemd, "register_service", lambda *_args, **_kwargs: None)
    monkeypatch.setattr(compose.site_systemd, "enable_and_start", lambda *_args, **_kwargs: None)
    monkeypatch.setattr(compose.runtime, "setup", lambda *_args, **_kwargs: (_ for _ in ()).throw(AssertionError()))

    compose.deploy(ctx)


def test_compose_service_waits_for_the_first_release():
    template = read(SRC_DIR / "bonesinfra/assets/systemd/compose.service.j2")

    assert "ConditionPathExists={{ paths.current }}/compose.yaml" in template


def test_compose_manifest_omits_nginx_without_a_loopback_port():
    ctx = DeployContext.from_request(make_site_request(backend="docker", domain="", ssl_enabled=False))

    assert all("nginx" not in artifact[0].lower() for artifact in compose.artifacts(ctx))


def test_docker_repository_accepts_supported_distributions_and_rejects_unknown_hosts():
    assert compose._docker_repository({"release_meta": {"ID": "debian", "VERSION_CODENAME": "bookworm"}}) == (
        "debian",
        "bookworm",
    )
    assert compose._docker_repository({"release_meta": {"ID": "ubuntu", "VERSION_CODENAME": "noble"}}) == (
        "ubuntu",
        "noble",
    )

    try:
        compose._docker_repository({"release_meta": {"ID": "fedora", "VERSION_CODENAME": "forty"}})
    except ValueError as error:
        assert "Debian or Ubuntu" in str(error)
    else:
        raise AssertionError("unsupported distributions must be rejected")


def test_compose_migration_removes_all_native_runtime_and_legacy_docker_artifacts(monkeypatch):
    ctx = DeployContext.from_request(
        make_site_request(backend="docker", project_name="example", domain="", ssl_enabled=False)
    )
    shell_calls = []
    monkeypatch.setattr(compose.server, "shell", lambda **kwargs: shell_calls.append(kwargs))
    monkeypatch.setattr(compose.systemd, "daemon_reload", lambda **_kwargs: None)
    monkeypatch.setattr(compose.systemd, "service", lambda **_kwargs: None)

    compose._remove_stale_managed_runtime(ctx)

    commands = "\n".join(command for call in shell_calls for command in call["commands"])
    for service in ("gunicorn", "puma"):
        assert f"example-{service}.service" in commands
        assert ctx.paths.apparmor_profile(service) in commands
    assert ctx.paths.site_nginx_config in commands
    assert "/var/lib/bonesdeploy/runtime-images/example" in commands
    assert "/etc/php/*/fpm/pool.d/example.conf" in commands
    assert "/run/example/php-fpm.sock" in commands
