from bonesinfra.config.context import DeployContext
from bonesinfra.services.linux import cloudflared, etckeeper


def deploy_tunnel_start(ctx: DeployContext):
    cloudflared.start(ctx, ctx.paths_dict)
    etckeeper.commit_changes("BonesInfra Quick Tunnel start")


def deploy_tunnel_stop(ctx: DeployContext):
    cloudflared.stop(ctx, ctx.paths_dict)
    etckeeper.commit_changes("BonesInfra Quick Tunnel stop")
