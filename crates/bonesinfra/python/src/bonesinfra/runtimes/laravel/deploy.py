from pathlib import Path

from bonesinfra.runtimes.laravel import nginx, php_fpm, php_packages, php_repo


def deploy(ctx):
    here = Path(__file__).parent
    php_version = ctx.runtime.data.get("php_version", "8.5")
    paths = ctx.paths_dict

    php_repo.add_php_apt_source()
    php_packages.install_php(php_version)

    php_fpm.setup_pool(here, ctx, paths, php_version)
    nginx.setup(here, ctx, paths, php_version)
