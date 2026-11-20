from pyinfra.operations import apt

BASE_SYSTEM_PACKAGES: list[str] = [
    "build-essential",
    "borgbackup",
    "ca-certificates",
    "fail2ban",
    "curl",
    "git",
    "gpg",
    "rsync",
    "sudo",
    "nginx",
    "openssl",
    "apparmor",
    "apparmor-utils",
    "unattended-upgrades",
    "certbot",
    "ufw",
    "etckeeper",
]

SUPPLEMENTARY_PACKAGES: list[str] = [
    "acl",
    "htop",
    "nano",
    "sqlite3",
    "unzip",
    "zip",
    "zsh",
]


def install_system(packages):
    apt.packages(
        name="Install setup apt packages",
        packages=packages,
        present=True,
        update=True,
        cache_time=3600,
        _sudo=True,
    )
