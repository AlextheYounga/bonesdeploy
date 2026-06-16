import os

from pyinfra import host
from pyinfra.facts.server import LinuxDistribution
from pyinfra.operations import apt, files, server
from src.utils import unflatten


BASE_SYSTEM_PACKAGES = [
    "build-essential",
    "ca-certificates",
    "curl",
    "git",
    "rsync",
    "sudo",
    "nginx",
    "apparmor",
    "apparmor-utils",
    "certbot",
    "ufw",
]


def deploy():
    data = unflatten(host.data.dict())
    paths = data.get("paths", {})
    here = os.path.dirname(__file__)

    packages = data.get("setup_apt_packages", BASE_SYSTEM_PACKAGES)
    install_system_packages(packages)
    install_rust()
    ensure_users_and_groups(data, paths)
    setup_repo_and_project_dirs(data, paths)
    setup_placeholder_release(data, paths, here)
    install_authorized_key(data)
    setup_firewall(data)
    install_bonesremote(data)


def install_system_packages(packages):
    apt.packages(
        name="Install setup apt packages",
        packages=packages,
        present=True,
        update=True,
        cache_time=3600,
        _sudo=True,
    )


def install_rust():
    cargo_bin = "/root/.cargo/bin/cargo"
    server.shell(
        name="Install rustup and cargo",
        commands=[
            "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal"
        ],
        _sudo=True,
    )


def ensure_users_and_groups(data, paths):
    server.user(
        name="Ensure deploy user exists",
        user=data["deploy_user"],
        shell="/bin/bash",
        ensure_home=True,
        _sudo=True,
    )

    server.user(
        name="Ensure runtime user exists",
        user=data["runtime_user"],
        system=True,
        home="/nonexistent",
        shell="/usr/sbin/nologin",
        create_home=False,
        _sudo=True,
    )

    server.group(
        name="Ensure runtime group exists",
        group=data["runtime_group"],
        _sudo=True,
    )

    server.user(
        name="Ensure runtime user is in runtime group",
        user=data["runtime_user"],
        groups=[data["runtime_group"]],
        append=True,
        _sudo=True,
    )

    server.group(
        name="Ensure release-read group exists",
        group=data["release_group"],
        _sudo=True,
    )

    server.user(
        name="Ensure runtime user is in release-read group",
        user=data["runtime_user"],
        groups=[data["release_group"]],
        append=True,
        _sudo=True,
    )


def setup_repo_and_project_dirs(data, paths):
    files.directory(
        name="Ensure bare repo parent directory exists",
        path=paths["repo_parent"],
        user=data["deploy_user"],
        group=data["deploy_user"],
        mode="0755",
        _sudo=True,
    )

    server.shell(
        name="Initialize bare git repo",
        commands=[f"git init --bare {paths['repo']}"],
        _sudo=True,
        _sudo_user=data["deploy_user"],
    )

    files.directory(
        name="Ensure bare repo bones directory exists",
        path=paths["repo_bones"],
        user=data["deploy_user"],
        group=data["deploy_user"],
        mode="0755",
        _sudo=True,
    )

    files.directory(
        name="Ensure project root parent directory is traversable",
        path=paths["project_root_parent"],
        user="root",
        group="root",
        mode="0711",
        _sudo=True,
    )

    files.directory(
        name="Ensure project root with setgid for release group",
        path=data["project_root"],
        user=data["deploy_user"],
        group=data["release_group"],
        mode="2751",
        _sudo=True,
    )

    files.directory(
        name="Ensure releases directory with setgid",
        path=paths["releases"],
        user=data["deploy_user"],
        group=data["release_group"],
        mode="2750",
        _sudo=True,
    )

    files.directory(
        name="Ensure build directory (private to deploy user)",
        path=os.path.join(data["project_root"], "build"),
        user=data["deploy_user"],
        group=data["deploy_user"],
        mode="0700",
        _sudo=True,
    )

    files.directory(
        name="Ensure shared directory (owned by runtime user)",
        path=paths["shared"],
        user=data["runtime_user"],
        group=data["runtime_group"],
        mode="0711",
        _sudo=True,
    )

    files.directory(
        name="Ensure placeholder release directory exists",
        path=paths["placeholder_web_root"],
        user=data["deploy_user"],
        group=data["release_group"],
        mode="0750",
        _sudo=True,
    )


def setup_placeholder_release(data, paths, here):
    files.template(
        name="Seed placeholder index page",
        src=os.path.join(here, "assets/nginx/index.html.j2"),
        dest=paths["placeholder_index"],
        user=data["deploy_user"],
        group=data["release_group"],
        mode="0640",
        **data,
        _sudo=True,
    )

    files.link(
        name="Point current symlink at placeholder release",
        path=paths["current"],
        target=paths["placeholder_release"],
        force=True,
        _sudo=True,
    )


def install_authorized_key(data):
    if not data.get("deploy_authorized_key"):
        return
    server.user(
        name="Ensure deploy user authorized key is installed",
        user=data["deploy_user"],
        public_keys=[data["deploy_authorized_key"]],
        _sudo=True,
    )


def setup_firewall(data):
    if not data.get("firewall_enabled", True):
        return

    ssh_port = int(data.get("ssh_port", 22))
    allowed_ports = data.get("firewall_allowed_ports", ["http", "https"])
    port_aliases = data.get("firewall_port_aliases", {"http": 80, "https": 443})
    rate_limit = data.get("firewall_ssh_rate_limit", False)
    ssh_cidrs = data.get("firewall_ssh_allowed_cidrs", [])
    manage_ssh = data.get("firewall_manage_ssh", True)

    cmds = []

    if manage_ssh:
        rule = "limit" if rate_limit else "allow"
        if not ssh_cidrs:
            cmds.append(f"ufw {rule} {ssh_port}/tcp")
        else:
            for cidr in ssh_cidrs:
                cmds.append(f"ufw {rule} from {cidr} to any port {ssh_port} proto tcp")

    for port in allowed_ports:
        if port == "ssh":
            continue
        port_num = port_aliases.get(port, port)
        cmds.append(f"ufw allow {port_num}/tcp")

    cmds.append(f"ufw --force default {data.get('firewall_default_incoming_policy', 'deny')} incoming")
    cmds.append(f"ufw --force default {data.get('firewall_default_outgoing_policy', 'allow')} outgoing")
    cmds.append("ufw --force enable")

    server.shell(
        name="Apply UFW configuration",
        commands=cmds,
        _sudo=True,
    )

    if data.get("firewall_show_status", True):
        server.shell(
            name="Display UFW status",
            commands=["ufw status verbose"],
            _sudo=True,
        )


def install_bonesremote(data):
    cargo_bin = "/root/.cargo/bin/cargo"
    server.shell(
        name="Install bonesremote binary",
        commands=[
            f"{cargo_bin} install --root /usr/local --git https://github.com/AlextheYounga/bonesdeploy.git bonesremote"
        ],
        _sudo=True,
    )

    server.shell(
        name="Run bonesremote init",
        commands=["/usr/local/bin/bonesremote init"],
        _sudo=True,
    )


deploy()
