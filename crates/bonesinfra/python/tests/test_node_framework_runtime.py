from types import SimpleNamespace

import pytest

from bonesinfra.config.context import DeployContext
from bonesinfra.frameworks.next import manifest as next_manifest, runtime as next_runtime
from bonesinfra.frameworks.nuxt import manifest as nuxt_manifest, runtime as nuxt_runtime
from bonesinfra.frameworks.sveltekit import manifest as sveltekit_manifest, runtime as sveltekit_runtime

from .helpers import make_site_request

NODE_BINARY = "/srv/sites/example/.bonesdeploy/runtimes/node/bin/node"
PATHS = {
    "current": "/srv/sites/example/current",
    "placeholder_release": "/srv/sites/example/releases/placeholder",
    "runtime_socket_dir": "/run/bonesdeploy/example",
}


@pytest.mark.parametrize(
    ("framework", "entrypoint", "command"),
    [
        (next_runtime, ".next/standalone/server.js", f"{NODE_BINARY} .next/standalone/server.js"),
        (nuxt_runtime, ".output/server/index.mjs", f"{NODE_BINARY} .output/server/index.mjs"),
        (sveltekit_runtime, "build/index.js", f"{NODE_BINARY} --env-file=.env build"),
    ],
)
def test_dynamic_node_frameworks_use_the_managed_site_link_for_services_and_validation(
    monkeypatch, framework, entrypoint, command
):
    captured = {}
    validations = []
    ctx = SimpleNamespace(
        app=SimpleNamespace(dns=SimpleNamespace(domain="example.test")),
        runtime=SimpleNamespace(data={"is_static": False}, runtime_group="example"),
        paths_dict={},
    )

    monkeypatch.setattr(framework.runtime, "orchestrate", lambda _ctx, provision, **_kwargs: provision(ctx))
    monkeypatch.setattr(framework.shared, "ensure_directories", lambda *_args: None)
    monkeypatch.setattr(framework.application, "deploy_server", lambda _ctx, **kwargs: captured.update(kwargs))
    monkeypatch.setattr(
        framework.validation,
        "run_as_runtime_user",
        lambda *args, **_kwargs: validations.append(args),
    )

    framework.deploy(ctx)

    runtime_command = captured["command"](ctx, PATHS, NODE_BINARY)
    assert command in runtime_command
    assert "mise" not in runtime_command
    assert "shims" not in runtime_command
    assert captured["exec_paths"](ctx, PATHS, NODE_BINARY) == [NODE_BINARY]
    captured["validate"](ctx, PATHS, NODE_BINARY)
    assert validations[0][2] == f"{NODE_BINARY} --check /srv/sites/example/current/{entrypoint}"


@pytest.mark.parametrize("project_manifest", [next_manifest, nuxt_manifest, sveltekit_manifest])
def test_dynamic_node_framework_manifests_declare_the_managed_site_link(project_manifest):
    ctx = DeployContext.from_request(make_site_request(project_name="example"))
    ctx.runtime.data["is_static"] = False

    artifacts = {name: (path, kind) for name, path, kind, _owner in project_manifest.artifacts(ctx)}

    assert artifacts["managed Node runtime"] == ("/srv/sites/example/.bonesdeploy/runtimes/node", "link")


@pytest.mark.parametrize("project_manifest", [next_manifest, nuxt_manifest])
def test_static_node_framework_manifests_do_not_declare_a_production_node_runtime(project_manifest):
    ctx = DeployContext.from_request(make_site_request(project_name="example"))

    artifact_names = {name for name, _path, _kind, _owner in project_manifest.artifacts(ctx)}

    assert "managed Node runtime" not in artifact_names
