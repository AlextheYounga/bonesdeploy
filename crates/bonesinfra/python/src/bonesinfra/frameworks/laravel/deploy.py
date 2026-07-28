from pathlib import Path

from bonesinfra.frameworks.base import PHPFramework
from bonesinfra.frameworks.laravel import php_packages, php_repo


class LaravelFramework(PHPFramework):
    assets_dir = Path(__file__).parent
    nginx_template = "assets/nginx/laravel-site-nginx.conf.j2"

    def install_packages(self, ctx):
        php_version = self.php_version(ctx)
        php_repo.add_php_apt_source()
        php_packages.install_php(php_version)


FRAMEWORK = LaravelFramework()
