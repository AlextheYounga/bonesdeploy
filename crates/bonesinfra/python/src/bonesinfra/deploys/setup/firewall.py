from pyinfra.operations import server


def configure(ctx):
    if not ctx.runtime.data.get("firewall_enabled", True):
        return

    ssh_port = int(ctx.runtime.data.get("ssh_port", int(ctx.app.server.port)))
    allowed_ports = ctx.runtime.data.get("firewall_allowed_ports", ["http", "https"])
    port_aliases = ctx.runtime.data.get("firewall_port_aliases", {"http": 80, "https": 443})
    rate_limit = ctx.runtime.data.get("firewall_ssh_rate_limit", False)
    ssh_cidrs = ctx.runtime.data.get("firewall_ssh_allowed_cidrs", [])
    manage_ssh = ctx.runtime.data.get("firewall_manage_ssh", True)

    cmds = []

    if manage_ssh:
        rule = "limit" if rate_limit else "allow"
        if not ssh_cidrs:
            cmds.append(f"ufw {rule} {ssh_port}/tcp")
        else:
            cmds.extend(f"ufw {rule} from {cidr} to any port {ssh_port} proto tcp" for cidr in ssh_cidrs)

    for port in allowed_ports:
        if port == "ssh":
            continue
        port_num = port_aliases.get(port, port)
        cmds.append(f"ufw allow {port_num}/tcp")

    incoming = ctx.runtime.data.get("firewall_default_incoming_policy", "deny")
    outgoing = ctx.runtime.data.get("firewall_default_outgoing_policy", "allow")
    cmds.append(f"ufw --force default {incoming} incoming")
    cmds.append(f"ufw --force default {outgoing} outgoing")
    cmds.append("ufw --force enable")

    server.shell(
        name="Apply UFW configuration",
        commands=cmds,
        _sudo=True,
    )

    if ctx.runtime.data.get("firewall_show_status", True):
        server.shell(
            name="Display UFW status",
            commands=["ufw status verbose"],
            _sudo=True,
        )
