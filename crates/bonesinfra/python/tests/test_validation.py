from bonesinfra.config.context import DeployContext
from bonesinfra.services.linux import validation

from .helpers import make_site_request


def test_runtime_user_validation_uses_the_current_release_as_its_working_directory(monkeypatch):
    calls = []
    monkeypatch.setattr(validation.server, "shell", lambda **kwargs: calls.append(kwargs))
    ctx = DeployContext.from_request(make_site_request())

    validation.run_as_runtime_user(ctx, "Validate runtime", "true")

    assert calls[0]["_chdir"] == ctx.paths.current


def test_verify_profile_attached_retries_before_diagnostics(monkeypatch):
    calls = []
    monkeypatch.setattr(validation.server, "shell", lambda **kwargs: calls.append(kwargs))

    validation.verify_profile_attached("shop-next.service", "bonesdeploy-shop-next")

    command = calls[0]["commands"][0]
    assert f'"$attempt" -lt {validation.PROFILE_CHECK_ATTEMPTS}' in command
    assert f"sleep {validation.PROFILE_CHECK_INTERVAL_SECONDS}" in command
    assert "systemctl is-active --quiet shop-next.service" in command
    assert "grep -qF -- bonesdeploy-shop-next /proc/$pid/attr/current" in command
    assert "systemctl status shop-next.service --no-pager --full" in command
    assert "journalctl -u shop-next.service -n 50 --no-pager" in command


def test_verify_profile_attached_supports_custom_operation_name(monkeypatch):
    calls = []
    monkeypatch.setattr(validation.server, "shell", lambda **kwargs: calls.append(kwargs))

    validation.verify_profile_attached("shop-next.service", "bonesdeploy-shop-next", name="Check Nuxt profile")

    assert calls[0]["name"] == "Check Nuxt profile"


def test_supported_host_policy_accepts_the_minimum_supported_versions():
    validation.validate_supported_host_facts({"release_meta": {"ID": "debian", "VERSION_ID": "12"}}, "x86_64")
    validation.validate_supported_host_facts({"release_meta": {"ID": "ubuntu", "VERSION_ID": "24.04"}}, "x86_64")


def test_supported_host_policy_accepts_newer_supported_versions():
    validation.validate_supported_host_facts({"release_meta": {"ID": "debian", "VERSION_ID": "13.1"}}, "x86_64")
    validation.validate_supported_host_facts({"release_meta": {"ID": "ubuntu", "VERSION_ID": "24.10"}}, "x86_64")


def test_supported_host_policy_rejects_unsupported_or_malformed_hosts():
    unsupported_hosts = [
        ({"release_meta": {"ID": "debian", "VERSION_ID": "11"}}, "x86_64"),
        ({"release_meta": {"ID": "ubuntu", "VERSION_ID": "24"}}, "x86_64"),
        ({"release_meta": {"ID": "Ubuntu", "VERSION_ID": "24.04"}}, "x86_64"),
        ({"release_meta": {"ID": "fedora", "VERSION_ID": "42"}}, "x86_64"),
        ({"release_meta": {"ID": "debian", "VERSION_ID": "12-bookworm"}}, "x86_64"),
        ({"release_meta": {"ID": "debian"}}, "x86_64"),
        ({}, "x86_64"),
        (None, "x86_64"),
        ({"release_meta": {"ID": "debian", "VERSION_ID": "12"}}, "aarch64"),
    ]

    for distribution, architecture in unsupported_hosts:
        try:
            validation.validate_supported_host_facts(distribution, architecture)
        except ValueError as error:
            assert str(error) == validation.SUPPORTED_HOST_MESSAGE
        else:
            raise AssertionError("unsupported production hosts must be rejected")
