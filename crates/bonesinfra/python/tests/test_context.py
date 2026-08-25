"""Deploy context parsing for the root dotenv contract."""

from pathlib import Path

import pytest

from bonesinfra.config.context import DeployContext, ServerContext, template_data


def _write_config(tmp_path: Path, extra: str = "") -> Path:
    path = tmp_path / ".env"
    path.write_text(
        f"""PROJECT_NAME=lawsnipe
HOST=example.com
PORT=2222
DOMAIN=example.com
PREVIEW_DOMAIN=preview.example.com
EMAIL=ops@example.com
SSL_ENABLED=true
BRANCH=main
WEB_ROOT=dist
{extra}"""
    )
    return path


def test_reads_nested_single_file_config(tmp_path):
    ctx = DeployContext.from_files(str(_write_config(tmp_path)))

    assert ctx.app.project_name == "lawsnipe"
    assert ctx.app.repo_path == "/home/git/lawsnipe.git"
    assert ctx.app.project_root == "/srv/sites/lawsnipe"
    assert ctx.server.host == "example.com"
    assert ctx.server.port == "2222"
    assert ctx.paths.repo == "/home/git/lawsnipe.git"
    assert ctx.paths.project_root == "/srv/sites/lawsnipe"
    assert ctx.paths.current_web_root == "/srv/sites/lawsnipe/current/dist"
    assert ctx.app.deploy.branch == "main"
    assert ctx.app.dns.ssl_enabled is True
    assert ctx.runtime.runtime_user == "lawsnipe"
    assert ctx.runtime.runtime_group == "lawsnipe"


def test_template_data_contains_runtime_values(tmp_path):
    td = template_data(DeployContext.from_files(str(_write_config(tmp_path))))
    assert td["runtime_user"] == "lawsnipe"
    assert td["runtime_group"] == "lawsnipe"
    assert td["runtime_backend"] == "native"


def test_framework_values_are_available_to_runtime_templates(tmp_path):
    ctx = DeployContext.from_files(_write_config(tmp_path, "IS_STATIC=true\n"))

    assert ctx.runtime.data["is_static"] is True
    assert template_data(ctx)["is_static"] is True


def test_false_framework_boolean_is_false(tmp_path):
    ctx = DeployContext.from_files(_write_config(tmp_path, "IS_STATIC=false\n"))

    assert ctx.runtime.data["is_static"] is False


def test_docker_runtime_backend_is_preserved(tmp_path):
    ctx = DeployContext.from_files(_write_config(tmp_path, "RUNTIME_BACKEND=docker\n"))

    assert ctx.runtime.backend == "docker"
    assert "backend" not in ctx.runtime.data


def test_unknown_runtime_backend_is_rejected(tmp_path):
    with pytest.raises(ValueError, match="RUNTIME_BACKEND"):
        DeployContext.from_files(_write_config(tmp_path, "RUNTIME_BACKEND=compose\n"))


def test_missing_nested_tables_use_defaults(tmp_path):
    path = tmp_path / ".env"
    path.write_text("PROJECT_NAME=lawsnipe\n")
    ctx = DeployContext.from_files(str(path))
    assert ctx.app.deploy.branch == "main"
    assert ctx.server.host == ""
    assert ctx.app.repo_path == "/home/git/lawsnipe.git"
    assert ctx.app.project_root == "/srv/sites/lawsnipe"
    assert ctx.runtime.web_root == "public"
    assert ctx.runtime.runtime_user == "lawsnipe"


def test_database_services_are_read_and_validated(tmp_path):
    ctx = DeployContext.from_files(_write_config(tmp_path, "SERVICES=postgres,valkey\n"))
    assert ctx.services.services == ("postgres", "valkey")


def test_conflicting_mysql_implementations_are_rejected(tmp_path):
    path = _write_config(tmp_path, "SERVICES=mariadb,mysql\n")
    with pytest.raises(ValueError, match="cannot be provisioned together"):
        DeployContext.from_files(str(path))


def test_duplicate_database_services_are_rejected(tmp_path):
    path = _write_config(tmp_path, "SERVICES=postgres,postgres\n")
    with pytest.raises(ValueError, match="must not contain duplicates"):
        DeployContext.from_files(str(path))


def test_dotenv_parser_ignores_comments_and_unquotes_values(tmp_path):
    path = tmp_path / ".env"
    path.write_text('# deployment settings\nPROJECT_NAME="lawsnipe"\nHOST=example.com  \nCUSTOM=value\n')

    ctx = DeployContext.from_files(str(path))

    assert ctx.app.project_name == "lawsnipe"
    assert ctx.server.host == "example.com"
    assert ctx.runtime.data["custom"] == "value"


def test_dotenv_parser_rejects_malformed_entries(tmp_path):
    path = tmp_path / ".env"
    path.write_text("PROJECT_NAME=lawsnipe\nnot-an-entry\n")

    with pytest.raises(ValueError, match="line 2"):
        DeployContext.from_files(str(path))


@pytest.mark.parametrize("entry", ["1BAD=value", "BAD-KEY=value", "=value"])
def test_dotenv_parser_rejects_invalid_keys(tmp_path, entry):
    with pytest.raises(ValueError, match="line 1"):
        DeployContext.from_files(_write_config(tmp_path, entry + "\n"))


def test_dotenv_parser_rejects_duplicate_keys(tmp_path):
    with pytest.raises(ValueError, match="duplicate"):
        DeployContext.from_files(_write_config(tmp_path, "HOST=other.example.com\n"))


def test_dotenv_parser_preserves_quote_semantics(tmp_path):
    path = tmp_path / ".env"
    path.write_text("PROJECT_NAME='lawsnipe'\nCUSTOM=\"quoted value\"\nRAW='one\"two'\n")

    ctx = DeployContext.from_files(str(path))

    assert ctx.app.project_name == "lawsnipe"
    assert ctx.runtime.data == {"custom": "quoted value", "raw": 'one"two'}


@pytest.mark.parametrize("project_name", ["", "Demo", "demo_name", "network", "demo;rm"])
def test_project_identity_matches_remote_validation(tmp_path, project_name):
    path = tmp_path / ".env"
    path.write_text(f"PROJECT_NAME={project_name}\n")

    with pytest.raises(ValueError, match="Invalid project name"):
        DeployContext.from_files(str(path))


def test_server_context_uses_only_connection_values(tmp_path):
    path = _write_config(tmp_path, "INVALID_RUNTIME=value\n")

    ctx = ServerContext.from_files(str(path))

    assert (ctx.host, ctx.ssh_user, ctx.port) == ("example.com", "root", "2222")
