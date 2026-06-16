import os
from pyinfra import host
from pyinfra.facts.server import LinuxDistribution
from pyinfra.operations import apt, files, server, systemd
from src.utils import load_runtime_config, unflatten


PHP_SURY_KEYRING_URL = "https://packages.sury.org/debsuryorg-archive-keyring.deb"
PHP_SURY_KEYRING_PATH = "/tmp/debsuryorg-archive-keyring.deb"
PHP_SURY_KEYRING_DEST = "/usr/share/keyrings/deb.sury.org-php.gpg"
PHP_SURY_PREREQUISITES = [
    "apt-transport-https",
    "ca-certificates",
    "curl",
    "lsb-release",
]


def questions():
    return [
        {
            "key": "php_version",
            "type": "choice",
            "label": "PHP version",
            "choices": ["8.2", "8.3", "8.4"],
            "default": "8.3",
        },
        {
            "key": "install_queue_worker",
            "type": "bool",
            "label": "Install Laravel queue worker?",
            "default": False,
        },
    ]


def defaults():
    return {
        "php_version": "8.3",
        "install_queue_worker": False,
        "shared_paths": [
            {"path": ".env", "type": "file", "required": True},
            {"path": "storage", "type": "dir", "required": True},
        ],
    }


def shared_paths(ctx):
    return defaults()["shared_paths"]


def apply(ctx):
    raise NotImplementedError("laravel apply is not migrated yet")


def deploy():


    here = os.path.dirname(__file__)
    data = unflatten(host.data.dict())
    runtime = load_runtime_config(__file__)
    php_version = runtime.get("php_version", "8.3")
    project = data["project_name"]
    paths = data["paths"]
    runtime_user = data["runtime_user"]
    runtime_group = data["runtime_group"]

    pool_config_path = f"/srv/conf/{project}/php-fpm.conf"
    php_fpm_socket_path = paths["runtime_php_fpm_socket"]

    setup_php_repository(php_version)
    install_php_packages(php_version)

    setup_storage_directories(paths, runtime_user, runtime_group)
    setup_php_fpm(here, data, paths, project, php_version, pool_config_path, php_fpm_socket_path, runtime_user, runtime_group)
    setup_php_fpm_apparmor(project, here, data)
    setup_laravel_nginx(here, data, paths, project, php_fpm_socket_path, runtime_user, runtime_group)


def _resolve_codename():
    deb = host.get_fact(LinuxDistribution)
    release_meta = deb.get("release_meta", {}) if deb else {}
    return (
        release_meta.get("VERSION_CODENAME")
        or release_meta.get("CODENAME")
        or release_meta.get("DISTRIB_CODENAME")
        or "noble"
    )


def _add_php_apt_source(php_version):
    apt.packages(
        name="Install PHP repo prerequisites",
        packages=PHP_SURY_PREREQUISITES,
        present=True,
        update=True,
        _sudo=True,
    )

    server.shell(
        name="Download PHP repo keyring package",
        commands=[f"curl -sSLo {PHP_SURY_KEYRING_PATH} {PHP_SURY_KEYRING_URL}"],
        _sudo=True,
    )

    apt.deb(
        name="Install PHP repo keyring package",
        src=PHP_SURY_KEYRING_PATH,
        _sudo=True,
    )

    server.shell(
        name="Remove stale PHP apt source file",
        commands=["rm -f /etc/apt/sources.list.d/php.list"],
        _sudo=True,
    )

    codename = _resolve_codename()
    apt.repo(
        name="Add Laravel PHP apt repository",
        src=f"deb [signed-by={PHP_SURY_KEYRING_DEST}] https://packages.sury.org/php {codename} main",
        filename="php",
        _sudo=True,
    )


def setup_php_repository(php_version):
    _add_php_apt_source(php_version)


def install_php_packages(php_version):
    packages = [
        f"php{php_version}",
        f"php{php_version}-cli",
        f"php{php_version}-fpm",
        f"php{php_version}-bcmath",
        f"php{php_version}-curl",
        f"php{php_version}-gd",
        f"php{php_version}-intl",
        f"php{php_version}-mbstring",
        f"php{php_version}-mysql",
        f"php{php_version}-sqlite3",
        f"php{php_version}-xml",
        f"php{php_version}-zip",
        "composer",
    ]

    apt.packages(
        name="Install Laravel PHP packages",
        packages=packages,
        present=True,
        update=True,
        _sudo=True,
    )


def setup_storage_directories(paths, runtime_user, runtime_group):
    subdirs = ["logs", "framework/cache", "framework/sessions", "framework/views"]
    for subdir in subdirs:
        files.directory(
            name=f"Ensure storage/{subdir} exists",
            path=f"{paths['current']}/storage/{subdir}",
            user=runtime_user,
            group=runtime_group,
            mode="0775",
            _sudo=True,
        )


def setup_php_fpm(here, data, paths, project, php_version, pool_config_path, php_fpm_socket_path, runtime_user, runtime_group):
    files.directory(
        name="Ensure conf directory exists",
        path=paths["conf_root"],
        user="root",
        group=runtime_group,
        mode="0750",
        _sudo=True,
    )

    files.template(
        name="Deploy PHP-FPM pool config",
        src=os.path.join(here, "src/assets/php/php-fpm-pool.conf.j2"),
        dest=pool_config_path,
        user="root",
        group="root",
        mode="0644",
        laravel_php_fpm_pool_name=project,
        laravel_php_fpm_socket_path=php_fpm_socket_path,
        **data,
        _sudo=True,
    )

    files.template(
        name="Deploy PHP-FPM systemd service",
        src=os.path.join(here, "src/assets/php/site-php-fpm.service.j2"),
        dest=f"/etc/systemd/system/{project}-php-fpm.service",
        user="root",
        group="root",
        mode="0644",
        laravel_php_fpm_pool_config_path=pool_config_path,
        laravel_php_version_resolved=php_version,
        apparmor_profile_name=f"bonesdeploy-{project}-php-fpm",
        **data,
        _sudo=True,
    )

    server.shell(
        name="Validate PHP-FPM configuration",
        commands=[f"/usr/sbin/php-fpm{php_version} --test --fpm-config {pool_config_path}"],
        _sudo=True,
    )

    systemd.service(
        name="Enable and start per-project PHP-FPM service",
        service=f"{project}-php-fpm.service",
        enabled=True,
        running=True,
        daemon_reload=True,
        _sudo=True,
    )


def setup_php_fpm_apparmor(project, here, data):
    profile_name = f"bonesdeploy-{project}-php-fpm"
    profile_path = f"/etc/apparmor.d/{profile_name}"

    files.template(
        name="Deploy PHP-FPM AppArmor profile",
        src=os.path.join(here, "src/assets/php/site-php-fpm-profile.j2"),
        dest=profile_path,
        user="root",
        group="root",
        mode="0644",
        apparmor_profile_name=profile_name,
        **data,
        _sudo=True,
    )

    server.shell(
        name="Load PHP-FPM AppArmor profile",
        commands=[f"apparmor_parser -r -T -W {profile_path}"],
        _sudo=True,
    )

def setup_laravel_nginx(here, data, paths, project, php_fpm_socket_path, runtime_user, runtime_group):
    files.template(
        name="Deploy Laravel-specific per-site nginx config",
        src=os.path.join(here, "src/assets/nginx/laravel-site-nginx.conf.j2"),
        dest=paths["site_nginx_config"],
        user="root",
        group=runtime_group,
        mode="0640",
        laravel_php_fpm_socket_path=php_fpm_socket_path,
        **data,
        _sudo=True,
    )

    files.directory(
        name="Ensure runtime socket directory exists before nginx validation",
        path=paths["runtime_socket_dir"],
        user=runtime_user,
        group=runtime_group,
        mode="0750",
        _sudo=True,
    )

    server.shell(
        name="Validate nginx configuration with Laravel config",
        commands=[f"nginx -t -c {paths['site_nginx_config']} -g 'daemon off;'"],
        _sudo=True,
    )

    systemd.service(
        name="Restart per-site nginx with Laravel config",
        service=f"{project}-nginx",
        restarted=True,
        _sudo=True,
    )
