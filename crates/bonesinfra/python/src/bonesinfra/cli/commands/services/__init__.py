from types import ModuleType

from bonesinfra.services.runtime import get_service


def deploy_services(ctx, custom: ModuleType | None = None):
    del custom
    for name in ctx.services.services:
        get_service(name).provision(ctx)
