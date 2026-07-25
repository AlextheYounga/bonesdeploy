from types import ModuleType

from bonesinfra.deploys.runtime import apparmor, nginx, packages, template_runtime
from bonesinfra.domain.custom import call_hook
from bonesinfra.runtimes import get_runtime


def deploy_runtime(ctx, custom: ModuleType | None = None):
    paths = ctx.paths_dict
    packages.install_apt(ctx)

    template = ctx.runtime.data.get("template")
    uses_tcp = False
    if template:
        runtime_mod = get_runtime(template)
        uses_tcp = getattr(runtime_mod, "USES_TCP", False) and not ctx.runtime.data.get("is_static", True)

    nginx_apparmor_network = "network inet stream," if uses_tcp else "network unix stream,"
    nginx_address_families = "AF_UNIX AF_INET" if uses_tcp else "AF_UNIX"

    apparmor.setup(ctx, paths, nginx_apparmor_network=nginx_apparmor_network)
    nginx.setup(ctx, paths, nginx_address_families=nginx_address_families, nginx_ip_loopback_only=uses_tcp)
    template_runtime.load(ctx)
    nginx.start_services(ctx, paths)

    call_hook(custom, "after_runtime", ctx)
