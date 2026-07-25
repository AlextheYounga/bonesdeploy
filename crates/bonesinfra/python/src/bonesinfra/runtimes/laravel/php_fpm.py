from pyinfra.operations import server

from bonesinfra.domain.paths import SCRIPTS_DIR
from bonesinfra.runtimes.common import php_fpm_pool


def cleanup_orphaned_pools(ctx, php_version):
    project = ctx.app.project_name
    current_pool = php_fpm_pool.pool_config_path(project, php_version)
    server.script_template(
        name="Remove orphaned Laravel PHP-FPM pools from other PHP versions",
        src=str(SCRIPTS_DIR / "cleanup-orphaned-php-pools.sh.j2"),
        project=project,
        current_pool=current_pool,
        _sudo=True,
    )


def setup_pool(here, ctx, paths, php_version):
    php_fpm_pool.ensure_log_dir(ctx)
    cleanup_orphaned_pools(ctx, php_version)
    php_fpm_pool.render_pool(ctx, here=here, paths=paths, php_version=php_version)
    php_fpm_pool.validate_php_fpm(php_version)
    php_fpm_pool.reload_php_fpm(php_version)
