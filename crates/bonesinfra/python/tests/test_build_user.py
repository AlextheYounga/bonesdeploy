import pytest

from bonesinfra.deploys.setup.users import (
    build_cache_for,
    build_group_for,
    build_home_for,
    build_user_for,
    cpu_quota_for,
)


def test_build_identity_is_derived_from_project_name():
    assert build_user_for("demo") == "demo-build"
    assert build_group_for("demo") == "demo-build"
    assert build_home_for("demo") == "/var/lib/bonesdeploy/users/demo-build"
    assert build_cache_for("demo") == "/var/lib/bonesdeploy/users/demo-build/cache"


@pytest.mark.parametrize(
    ("online_cpu_count", "quota"),
    [(1, "80%"), (2, "160%"), (4, "320%"), (8, "640%")],
)
def test_cpu_quota_is_default_percentage_of_online_cpu_capacity(online_cpu_count, quota):
    assert cpu_quota_for(online_cpu_count) == quota


def test_cpu_quota_rejects_missing_online_cpus():
    with pytest.raises(ValueError, match="positive"):
        cpu_quota_for(0)


def test_cpu_quota_uses_configured_per_cpu_percentage():
    assert cpu_quota_for(4, 50) == "200%"
