from pyinfra.operations import apt, files, server, systemd

# Install Laravel PHP repository prerequisites
apt.packages(
    name="Install PHP repo prerequisites",
    packages=data["laravel_php_sury_prerequisite_packages"],
    present=True,
    update_cache=True,
    _sudo=True,
)

server.shell(
    name="Download PHP repo keyring package",
    commands=[
        f"curl -sSLo {data['laravel_php_sury_keyring_package_path']} {data['laravel_php_sury_keyring_package_url']}"
    ],
    _sudo=True,
)

apt.deb(
    name="Install PHP repo keyring package",
    src=data["laravel_php_sury_keyring_package_path"],
    _sudo=True,
)

deb = host.get_fact(server.LinuxDistribution)
codename = deb.get("release_meta", {}).get("DISTRIB_CODENAME", "noble") if deb else "noble"

apt.repo(
    name="Add Laravel PHP apt repository",
    src=f"deb [signed-by={data['laravel_php_sury_keyring_path']}] https://packages.sury.org/php {codename} main",
    filename="php",
    _sudo=True,
)
