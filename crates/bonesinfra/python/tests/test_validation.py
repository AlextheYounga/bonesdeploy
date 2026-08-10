from bonesinfra.services.linux import validation


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
