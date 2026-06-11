import io
import os
import importlib.util

from pyinfra import host
from pyinfra.facts.server import LinuxDistribution
from pyinfra.operations import apt, files, server
from utils import unflatten

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


def _unflatten(data_dict):
    result = {}
    for key, value in data_dict.items():
        parts = key.split(".")
        node = result
        for part in parts[:-1]:
            if part not in node:
                node[part] = {}
            node = node[part]
        node[parts[-1]] = value
    return result


DEPLOY_DATA = _unflatten(host.data.dict())
PATHS = DEPLOY_DATA.get("paths", {})


def _load_optional_module(module_path, module_name):
    if os.path.exists(module_path):
        spec = importlib.util.spec_from_file_location(module_name, module_path)
        mod = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(mod)


# Template-specific pre-package setup
pre_packages_path = os.path.join(os.path.dirname(__file__), "pre_packages.py")
if os.path.exists(pre_packages_path):
    _load_optional_module(pre_packages_path, "pre_packages")

# Install setup apt packages
apt.packages(
    name="Install setup apt packages",
    packages=DEPLOY_DATA.get("setup_apt_packages", SETUP_APT_PACKAGES),
    present=True,
    update=True,
    cache_time=3600,
    _sudo=True,
)

# --- Common: bare repo and placeholder ---

files.directory(
    name="Ensure bare repo parent directory exists",
    path=PATHS["repo_parent"],
    user=DEPLOY_DATA["deploy_user"],
    group=DEPLOY_DATA["deploy_user"],
    mode="0755",
    _sudo=True,
)

server.shell(
    name="Initialize bare git repo",
    commands=[f"git init --bare {PATHS['repo']}"],
    _sudo=True,
    _sudo_user=DEPLOY_DATA["deploy_user"],
)

files.directory(
    name="Ensure bare repo bones directory exists",
    path=PATHS["repo_bones"],
    user=DEPLOY_DATA["deploy_user"],
    group=DEPLOY_DATA["deploy_user"],
    mode="0755",
    _sudo=True,
)

files.directory(
    name="Ensure project root parent directory is traversable",
    path=PATHS["project_root_parent"],
    user="root",
    group="root",
    mode="0711",
    _sudo=True,
)

files.directory(
    name="Ensure placeholder release directory exists",
    path=PATHS["placeholder_web_root"],
    user=DEPLOY_DATA["service_user"],
    group=DEPLOY_DATA["group"],
    mode="0750",
    _sudo=True,
)

placeholder_index = PATHS["placeholder_index"]
placeholder_html = f"""\
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{DEPLOY_DATA["project_name"]}</title>
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
        <h1>{DEPLOY_DATA["project_name"]}</h1>
        <p>{DEPLOY_DATA.get('setup_label', 'bonesdeploy')} deployment coming soon.</p>
    </main>
</body>
</html>"""

files.put(
    name="Seed placeholder index page",
    src=io.StringIO(placeholder_html),
    dest=placeholder_index,
    user=DEPLOY_DATA["service_user"],
    group=DEPLOY_DATA["group"],
    mode="0640",
    _sudo=True,
)

files.link(
    name="Point current symlink at placeholder release",
    path=PATHS["current"],
    target=PATHS["placeholder_release"],
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
    commands=[f"/usr/local/bin/bonesremote init --deploy-user {DEPLOY_DATA["deploy_user"]}"],
    _sudo=True,
)

# --- Users ---

server.user(
    name="Ensure deploy user exists",
    user=DEPLOY_DATA["deploy_user"],
    shell="/bin/bash",
    ensure_home=True,
    _sudo=True,
)

server.user(
    name="Ensure service user exists",
    user=DEPLOY_DATA["service_user"],
    system=True,
    home="/nonexistent",
    shell="/usr/sbin/nologin",
    create_home=False,
    _sudo=True,
)

server.group(
    name="Ensure service group exists",
    group=DEPLOY_DATA["group"],
    _sudo=True,
)

server.user(
    name="Ensure service user is in service group",
    user=DEPLOY_DATA["service_user"],
    groups=[DEPLOY_DATA["group"]],
    append=True,
    _sudo=True,
)

files.directory(
    name="Ensure web root exists",
    path=DEPLOY_DATA.get("live_root_parent", "/var/www"),
    user="root",
    group=DEPLOY_DATA["group"],
    mode="2775",
    _sudo=True,
)

if DEPLOY_DATA.get("deploy_authorized_key"):
    server.user(
        name="Ensure deploy user authorized key is installed",
        user=DEPLOY_DATA["deploy_user"],
        public_keys=[DEPLOY_DATA["deploy_authorized_key"]],
        _sudo=True,
    )

# --- Firewall ---

if DEPLOY_DATA.get("firewall_enabled", True):
    ssh_port = int(DEPLOY_DATA.get("ssh_port", 22))
    allowed_ports = DEPLOY_DATA.get("firewall_allowed_ports", ["http", "https"])
    port_aliases = DEPLOY_DATA.get("firewall_port_aliases", {"http": 80, "https": 443})
    rate_limit = DEPLOY_DATA.get("firewall_ssh_rate_limit", False)
    ssh_cidrs = DEPLOY_DATA.get("firewall_ssh_allowed_cidrs", [])
    manage_ssh = DEPLOY_DATA.get("firewall_manage_ssh", True)

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

    cmds.append(f"ufw --force default {DEPLOY_DATA.get('firewall_default_incoming_policy', 'deny')} incoming")
    cmds.append(f"ufw --force default {DEPLOY_DATA.get('firewall_default_outgoing_policy', 'allow')} outgoing")
    cmds.append("ufw --force enable")

    server.shell(
        name="Apply UFW configuration",
        commands=cmds,
        _sudo=True,
    )

if DEPLOY_DATA.get("firewall_show_status", True):
    server.shell(
        name="Display UFW status",
        commands=["ufw status verbose"],
        _sudo=True,
    )
