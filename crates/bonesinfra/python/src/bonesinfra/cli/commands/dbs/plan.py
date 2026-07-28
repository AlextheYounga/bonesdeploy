from types import ModuleType

from bonesinfra.services.runtime import catalog as services


def deploy_dbs(ctx, custom: ModuleType | None = None):
    del custom
    services.provision(ctx)
