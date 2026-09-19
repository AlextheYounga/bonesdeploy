"""Deployment paths should match the v1 host layout."""

from pathlib import Path

from bonesinfra.config import paths as config_paths
from bonesinfra.config.paths import DeploymentPaths


def test_paths_default_repo_parent_is_srv_git():
    paths = DeploymentPaths.new("lawsnipe", "/srv/git/lawsnipe.git", "/srv/sites/lawsnipe")

    assert paths.repo_parent == "/srv/git"


def test_paths_include_global_nginx_default_deny_site():
    paths = DeploymentPaths.new("lawsnipe", "/srv/git/lawsnipe.git", "/srv/sites/lawsnipe")

    assert paths.nginx_default_deny_site_available == "/etc/nginx/sites-available/00-bonesdeploy-default-deny.conf"
    assert paths.nginx_default_deny_site_enabled == "/etc/nginx/sites-enabled/00-bonesdeploy-default-deny.conf"
    assert paths.nginx_default_deny_ssl_certificate == "/etc/ssl/certs/bonesdeploy-default-deny.crt"
    assert paths.nginx_default_deny_ssl_certificate_key == "/etc/ssl/private/bonesdeploy-default-deny.key"


def test_paths_include_site_target_and_requires_directory():
    paths = DeploymentPaths.new("shop", "/srv/git/shop.git", "/srv/sites/shop")

    assert paths.systemd_site_target == "/etc/systemd/system/shop.target"
    assert paths.systemd_site_target_requires == "/etc/systemd/system/shop.target.requires"


def test_template_path_falls_back_to_package_when_project_templates_are_inaccessible(monkeypatch):
    def inaccessible_project(_path):
        raise PermissionError("project templates are inaccessible")

    monkeypatch.setattr(Path, "is_dir", inaccessible_project)

    template = config_paths.TEMPLATES_DIR / "frameworks" / "django" / "app.service.j2"

    assert str(template) == str(
        Path(config_paths.__file__).parent.parent / "frameworks/django/templates/app.service.j2"
    )
