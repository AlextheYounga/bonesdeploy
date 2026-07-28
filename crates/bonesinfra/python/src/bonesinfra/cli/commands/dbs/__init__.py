from types import ModuleType

from bonesinfra.services.runtime import get_service


def deploy_dbs(ctx, custom: ModuleType | None = None):
    del custom
    for name in ctx.dbs.services:
        get_service(name).provision(ctx)
