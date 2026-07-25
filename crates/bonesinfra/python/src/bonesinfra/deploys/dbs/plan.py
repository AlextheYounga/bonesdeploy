from types import ModuleType

from bonesinfra.deploys.dbs import services


def deploy_dbs(ctx, custom: ModuleType | None = None):
    del custom
    services.provision(ctx)
