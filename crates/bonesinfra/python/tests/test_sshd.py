"""Root password login is disabled by an sshd_config drop-in that wins over
the main sshd_config (first-obtained value wins), is validated before ssh is
reloaded, and removes itself instead of leaving an invalid sshd config."""

import types

import jinja2

from bonesinfra.cli.commands.server import sshd

from .helpers import SRC_DIR


def test_disable_root_password_login_renders_drop_in_and_reloads_ssh(monkeypatch):
    seen = {}

    def _capture_render(name, src, dest, **_data):
        seen["name"] = name
        seen["dest"] = dest
        return types.SimpleNamespace(changes=[])

    captured_commands = []

    def _capture_shell(name, commands, **_kwargs):
        seen.setdefault("shell_names", []).append(name)
        captured_commands.append(commands)
        return types.SimpleNamespace(changes=[])

    monkeypatch.setattr(sshd, "render", _capture_render)
    monkeypatch.setattr(sshd.server, "shell", _capture_shell)

    sshd.disable_root_password_login()

    assert seen["name"] == "Disable password login for root"
    assert seen["dest"] == sshd.ROOT_LOGIN_DROP_IN
    assert seen["dest"] == "/etc/ssh/sshd_config.d/99-bonesdeploy-root-login.conf"
    assert seen["shell_names"] == ["Validate sshd configuration", "Reload ssh to apply root login hardening"]
    assert captured_commands[0] == [
        "sshd -t || { rm -f /etc/ssh/sshd_config.d/99-bonesdeploy-root-login.conf; exit 1; }"
    ]
    assert captured_commands[1] == ["systemctl reload ssh"]


def test_root_login_drop_in_forbids_password_authentication():
    rendered = (
        jinja2.Environment(autoescape=True, loader=jinja2.FileSystemLoader(str(SRC_DIR / "bonesinfra/assets/sshd")))
        .get_template("99-bonesdeploy-root-login.conf.j2")
        .render()
    )
    assert "PermitRootLogin prohibit-password" in rendered
    assert "PermitRootLogin yes" not in rendered
