from bonesinfra.deploys._shared import nginx_safety


def test_validate_config_rejects_conflicting_server_name_warning(monkeypatch):
    calls = []

    def fake_script(*args, **kwargs):
        calls.append((args, kwargs))

    monkeypatch.setattr(nginx_safety.server, "script", fake_script)

    nginx_safety.validate_config("Validate nginx configuration")

    src = str(calls[0][1]["src"])
    assert "validate-nginx-safety.sh" in src


def test_install_default_deny_server_uses_dedicated_paths_and_disables_debian_default(monkeypatch, tmp_path):
    script_template_calls = []
    render_calls = []
    link_calls = []
    paths = {
        "nginx_default_deny_site_available": "/etc/nginx/sites-available/00-bonesdeploy-default-deny.conf",
        "nginx_default_deny_site_enabled": "/etc/nginx/sites-enabled/00-bonesdeploy-default-deny.conf",
        "nginx_default_deny_ssl_certificate": "/etc/ssl/certs/bonesdeploy-default-deny.crt",
        "nginx_default_deny_ssl_certificate_key": "/etc/ssl/private/bonesdeploy-default-deny.key",
        "nginx_default_site_enabled": "/etc/nginx/sites-enabled/default",
    }

    monkeypatch.setattr(nginx_safety, "ASSETS_DIR", tmp_path)

    def fake_script_template(*args, **kwargs):
        script_template_calls.append((args, kwargs))

    def fake_render(*args, **kwargs):
        render_calls.append((args, kwargs))

    def fake_link(*args, **kwargs):
        link_calls.append((args, kwargs))

    monkeypatch.setattr(nginx_safety.server, "script_template", fake_script_template)
    monkeypatch.setattr(nginx_safety, "render", fake_render)
    monkeypatch.setattr(nginx_safety.files, "link", fake_link)

    nginx_safety.install_default_deny_server(paths)

    call = script_template_calls[0][1]
    assert call["cert"] == paths["nginx_default_deny_ssl_certificate"]
    assert call["key"] == paths["nginx_default_deny_ssl_certificate_key"]
    assert render_calls[0][0][1] == tmp_path / "nginx/default-deny.conf.j2"
    assert render_calls[0][0][2] == paths["nginx_default_deny_site_available"]
    assert link_calls[0][1]["path"] == paths["nginx_default_deny_site_enabled"]
    assert link_calls[1][1]["path"] == paths["nginx_default_site_enabled"]
    assert link_calls[1][1]["present"] is False
