from bonesinfra.services.linux import etckeeper
from bonesinfra.services.runtime import get_service


def deploy_services(ctx):
    for name in ctx.services.services:
        get_service(name).provision(ctx)
    etckeeper.commit_changes("BonesInfra service provisioning")
