from types import ModuleType

from bonesinfra.cli.commands.runtime import template_runtime
from bonesinfra.cli.hooks import call_hook
from bonesinfra.frameworks import get_framework
from bonesinfra.services.linux.apparmor import nginx as apparmor
from bonesinfra.services.linux.nginx import router as nginx


def deploy_runtime(ctx, custom: ModuleType | None = None):
    paths = ctx.paths_dict

    template = ctx.runtime.data.get("template")
    uses_tcp = False
    if template:
        framework = get_framework(template)
        uses_tcp = framework.uses_tcp and not ctx.runtime.data.get("is_static", True)

    nginx_apparmor_network = "network inet stream," if uses_tcp else "network unix stream,"
    nginx_address_families = "AF_UNIX AF_INET" if uses_tcp else "AF_UNIX"

    apparmor.setup(ctx, paths, nginx_apparmor_network=nginx_apparmor_network)
    nginx.setup(ctx, paths, nginx_address_families=nginx_address_families, nginx_ip_loopback_only=uses_tcp)
    template_runtime.load(ctx)
    nginx.start_services(ctx, paths)

    call_hook(custom, "after_runtime", ctx)
