import os
import importlib.util

from pyinfra import host
from pyinfra.facts.server import LinuxDistribution
from pyinfra.operations import apt, files, server, systemd

SETUP_APT_PACKAGES = [
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


def _load_optional_module(module_path, module_name):
    if os.path.exists(module_path):
        spec = importlib.util.spec_from_file_location(module_name, module_path)
        mod = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(mod)


# Template-specific pre-package setup
if data.get("setup_pre_packages_enabled", False):
    _load_optional_module(
        os.path.join(os.path.dirname(__file__), "pre_packages.py"),
        "pre_packages",
    )

# Install setup apt packages
apt.packages(
    name="Install setup apt packages",
    packages=data.get("setup_apt_packages", SETUP_APT_PACKAGES),
    present=True,
    update_cache=True,
    cache_time=3600,
    _sudo=True,
)

# --- Common: bare repo and placeholder ---

files.directory(
    name="Ensure bare repo parent directory exists",
    path=data["paths"]["repo_parent"],
    user=data["deploy_user"],
    group=data["deploy_user"],
    mode="0755",
    _sudo=True,
)

server.shell(
    name="Initialize bare git repo",
    commands=[f"git init --bare {data['paths']['repo']}"],
    _sudo=True,
    _sudo_user=data["deploy_user"],
)

files.directory(
    name="Ensure bare repo bones directory exists",
    path=data["paths"]["repo_bones"],
    user=data["deploy_user"],
    group=data["deploy_user"],
    mode="0755",
    _sudo=True,
)

files.directory(
    name="Ensure project root parent directory is traversable",
    path=data["paths"]["project_root_parent"],
    user="root",
    group="root",
    mode="0711",
    _sudo=True,
)

files.directory(
    name="Ensure placeholder release directory exists",
    path=data["paths"]["placeholder_web_root"],
    user=data["service_user"],
    group=data["group"],
    mode="0750",
    _sudo=True,
)

placeholder_index = data["paths"]["placeholder_index"]
placeholder_html = f"""\
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{data['project_name']}</title>
    <style>
        body {{ font-family: system-ui, sans-serif; display: flex; justify-content: center;
               align-items: center; min-height: 100vh; margin: 0; background: #f5f5f5; }}
        main {{ text-align: center; padding: 2rem; }}
        h1 {{ color: #333; }}
        p {{ color: #666; }}
    </style>
</head>
<body>
    <main>
        <h1>{data['project_name']}</h1>
        <p>{data['setup_label']} deployment coming soon.</p>
    </main>
</body>
</html>"""

files.put(
    name="Seed placeholder index page",
    src=None,
    dest=placeholder_index,
    content=placeholder_html,
    user=data["service_user"],
    group=data["group"],
    mode="0640",
    _sudo=True,
)

files.link(
    name="Point current symlink at placeholder release",
    path=data["paths"]["current"],
    target=data["paths"]["placeholder_release"],
    force=True,
    _sudo=True,
)

# --- Common: rustup and bonesremote ---

rustup_bin = os.path.join("/root/.cargo/bin/rustup")
cargo_bin = os.path.join("/root/.cargo/bin/cargo")
br_bin = "/usr/local/bin/bonesremote"

deb_fact = host.get_fact(LinuxDistribution)
if deb_fact and deb_fact.get("name") == "Ubuntu":
    apt.packages(
        name="Install build-essential for bonesremote compilation",
        packages=["build-essential"],
        present=True,
        _sudo=True,
    )

server.shell(
    name="Install rustup and cargo",
    commands=[
        "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal"
    ],
    _sudo=True,
)

server.shell(
    name="Install bonesremote binary",
    commands=[
        f"{cargo_bin} install --root /usr/local --git https://github.com/AlextheYounga/bonesdeploy.git bonesremote"
    ],
    _sudo=True,
)

server.shell(
    name="Run bonesremote init",
    commands=[f"bonesremote init --deploy-user {data['deploy_user']}"],
    _env={"PATH": f"{data['paths']['usr_local_bin']}:{host.get_fact(server.Environment).get('PATH', '')}"},
    _sudo=True,
)

# --- Users ---

server.user(
    name="Ensure deploy user exists",
    user=data["deploy_user"],
    shell="/bin/bash",
    ensure_home=True,
    password_lock=True,
    _sudo=True,
)

server.user(
    name="Ensure service user exists",
    user=data["service_user"],
    system=True,
    home="/nonexistent",
    shell="/usr/sbin/nologin",
    _sudo=True,
)

server.group(
    name="Ensure service group exists",
    group=data["group"],
    _sudo=True,
)

server.user(
    name="Ensure service user is in service group",
    user=data["service_user"],
    groups=[data["group"]],
    _sudo=True,
)

files.directory(
    name="Ensure web root exists",
    path=data["live_root_parent"],
    user="root",
    group=data["group"],
    mode="2775",
    _sudo=True,
)

if data.get("deploy_authorized_key"):
    server.user(
        name="Ensure deploy user authorized key is installed",
        user=data["deploy_user"],
        public_keys=[data["deploy_authorized_key"]],
        _sudo=True,
    )

# --- Firewall ---

if data.get("firewall_enabled", True):
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
