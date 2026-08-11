from pathlib import Path
from types import SimpleNamespace

from bonesinfra.services.linux import runtime


def test_runtime_setup_configures_apparmor_and_nginx_for_unix_sockets(monkeypatch):
    calls = []
    ctx = SimpleNamespace(paths_dict={"runtime": "paths"})

    monkeypatch.setattr(runtime.apparmor, "setup", lambda *args, **kwargs: calls.append(("apparmor", args, kwargs)))
    monkeypatch.setattr(runtime.router, "setup", lambda *args, **kwargs: calls.append(("router", args, kwargs)))

    runtime.setup(ctx)

    assert [call[0] for call in calls] == ["apparmor", "router"]
    assert calls[0][2] == {"nginx_apparmor_network": "network unix stream,"}
    assert calls[1][2] == {"nginx_address_families": "AF_UNIX", "nginx_ip_loopback_only": False}


def test_runtime_setup_uses_tcp_settings_for_tcp_applications(monkeypatch):
    calls = []
    ctx = SimpleNamespace(paths_dict={"runtime": "paths"})

    monkeypatch.setattr(runtime.apparmor, "setup", lambda _ctx, _paths, **kwargs: calls.append(kwargs))
    monkeypatch.setattr(runtime.router, "setup", lambda _ctx, _paths, **kwargs: calls.append(kwargs))

    runtime.setup(ctx, uses_tcp=True)

    assert calls == [
        {"nginx_apparmor_network": "network inet stream,"},
        {"nginx_address_families": "AF_UNIX AF_INET", "nginx_ip_loopback_only": True},
    ]


def test_generated_runtimes_include_host_lifecycle_operations():
    assets = Path(__file__).parents[3] / "bonesdeploy" / "assets" / "frameworks"
    for runtime_source in sorted(assets.glob("*/infra/runtime.py")):
        source = runtime_source.read_text()
        assert "runtime.setup(ctx" in source, runtime_source
        assert "runtime.start_services(ctx)" in source, runtime_source
        assert source.index("runtime.setup(ctx") < source.index("runtime.start_services(ctx)"), runtime_source
