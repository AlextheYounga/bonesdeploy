from bonesinfra.cli.commands.site import directories, placeholder, users
from bonesinfra.config.context import DeployContext
from bonesinfra.services.linux import backup, etckeeper


def deploy_site_setup(ctx: DeployContext):
    paths = ctx.paths_dict
    users.ensure_users_and_groups(ctx)
    directories.setup_repo_and_project(ctx, paths)
    placeholder.seed(ctx, paths)
    backup.provision(ctx, paths)
    etckeeper.commit_changes("BonesInfra site setup")
