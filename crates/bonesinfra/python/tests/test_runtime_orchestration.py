from pathlib import Path
from types import SimpleNamespace

from bonesinfra.services.linux import runtime


def test_runtime_setup_configures_apparmor_and_nginx_for_unix_sockets(monkeypatch):
    calls = []
    ctx = SimpleNamespace(
        paths_dict={"runtime": "paths"}, app=SimpleNamespace(dns=SimpleNamespace(domain="example.test"))
    )

    monkeypatch.setattr(runtime.apparmor, "setup", lambda *args, **kwargs: calls.append(("apparmor", args, kwargs)))
    monkeypatch.setattr(runtime.router, "setup", lambda *args, **kwargs: calls.append(("router", args, kwargs)))

    runtime.setup(ctx)

    assert [call[0] for call in calls] == ["apparmor", "router"]
    assert calls[0][2] == {"nginx_apparmor_network": "network unix stream,"}
    assert calls[1][2] == {"nginx_address_families": "AF_UNIX", "nginx_ip_loopback_only": False}


def test_runtime_setup_uses_tcp_settings_for_tcp_applications(monkeypatch):
    calls = []
    ctx = SimpleNamespace(
        paths_dict={"runtime": "paths"}, app=SimpleNamespace(dns=SimpleNamespace(domain="example.test"))
    )

    monkeypatch.setattr(runtime.apparmor, "setup", lambda _ctx, _paths, **kwargs: calls.append(kwargs))
    monkeypatch.setattr(runtime.router, "setup", lambda _ctx, _paths, **kwargs: calls.append(kwargs))

    runtime.setup(ctx, uses_tcp=True)

    assert calls == [
        {"nginx_apparmor_network": "network inet stream,"},
        {"nginx_address_families": "AF_UNIX AF_INET", "nginx_ip_loopback_only": True},
    ]


def test_runtime_orchestrate_starts_services_after_provisioning(monkeypatch):
    calls = []
    ctx = SimpleNamespace(
        paths_dict={"runtime": "paths"}, app=SimpleNamespace(dns=SimpleNamespace(domain="example.test"))
    )

    monkeypatch.setattr(runtime, "setup", lambda *_args, **kwargs: calls.append(("setup", kwargs)))
    monkeypatch.setattr(runtime, "reconcile_ingress", lambda *_args: calls.append(("ingress", {})))
    monkeypatch.setattr(runtime, "start_services", lambda *_args: calls.append(("start", {})))

    runtime.orchestrate(ctx, lambda current_ctx: calls.append(("provision", current_ctx)), uses_tcp=True)

    assert [call[0] for call in calls] == ["setup", "provision", "ingress", "start"]
    assert calls[0][1] == {"uses_tcp": True}
    assert calls[1][1] is ctx


def test_runtime_reconcile_removes_router_before_setting_up_quick_tunnel(monkeypatch):
    calls = []
    ctx = SimpleNamespace(paths_dict={"runtime": "paths"}, app=SimpleNamespace(dns=SimpleNamespace(domain="")))

    monkeypatch.setattr(runtime.router, "remove_project_router", lambda paths: calls.append(("router-remove", paths)))
    monkeypatch.setattr(
        runtime.cloudflared,
        "setup",
        lambda current_ctx, paths: calls.append(("cloudflared-setup", current_ctx, paths)),
    )

    runtime.reconcile_ingress(ctx)

    assert [call[0] for call in calls] == ["router-remove", "cloudflared-setup"]
    assert calls[0][1] is ctx.paths_dict
    assert calls[1][1:] == (ctx, ctx.paths_dict)


def test_runtime_reconcile_removes_quick_tunnel_for_a_real_domain(monkeypatch):
    calls = []
    ctx = SimpleNamespace(
        paths_dict={"runtime": "paths"}, app=SimpleNamespace(dns=SimpleNamespace(domain="example.test"))
    )

    monkeypatch.setattr(runtime.cloudflared, "remove", lambda current_ctx, paths: calls.append((current_ctx, paths)))
    monkeypatch.setattr(runtime.router, "remove_project_router", lambda paths: calls.append(("router", paths)))
    monkeypatch.setattr(
        runtime.cloudflared,
        "setup",
        lambda current_ctx, paths: calls.append(("setup", current_ctx, paths)),
    )

    runtime.reconcile_ingress(ctx)

    assert calls == [(ctx, ctx.paths_dict)]


def test_generated_runtimes_include_host_lifecycle_operations():
    frameworks = Path(__file__).parents[1] / "src" / "bonesinfra" / "frameworks"
    for runtime_source in sorted(frameworks.glob("*/runtime.py")):
        if runtime_source.parent.name == "custom":
            continue
        source = runtime_source.read_text()
        assert "runtime.orchestrate(ctx, provision" in source, runtime_source


def test_generated_framework_runtimes_do_not_reach_through_to_project_hooks():
    frameworks = Path(__file__).parents[1] / "src" / "bonesinfra" / "frameworks"
    for runtime_source in sorted(frameworks.glob("*/runtime.py")):
        if runtime_source.parent.name == "custom":
            continue
        source = runtime_source.read_text()
        assert "custom.deploy(ctx)" not in source, runtime_source
