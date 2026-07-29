from bonesinfra.frameworks.base import PHPFramework


class LaravelFramework(PHPFramework):
    nginx_template = "nginx/laravel-site-nginx.conf.j2"


LARAVEL_FRAMEWORK = LaravelFramework()
