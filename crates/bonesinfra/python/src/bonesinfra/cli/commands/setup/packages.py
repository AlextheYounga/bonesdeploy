from pyinfra.operations import apt, server

BASE_SYSTEM_PACKAGES: list[str] = [
    "build-essential",
    "ca-certificates",
    "fail2ban",
    "curl",
    "dbus-user-session",
    "fuse-overlayfs",
    "git",
    "gpg",
    "rsync",
    "sudo",
    "nginx",
    "openssl",
    "aardvark-dns",
    "netavark",
    "passt",
    "podman",
    "slirp4netns",
    "uidmap",
    "apparmor",
    "apparmor-utils",
    "unattended-upgrades",
    "certbot",
    "ufw",
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


def install_rust():
    server.shell(
        name="Install rustup and cargo",
        commands=["curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal"],
        _sudo=True,
    )
