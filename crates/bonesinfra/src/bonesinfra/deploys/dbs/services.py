from pyinfra.operations import apt, server, systemd

from bonesinfra.domain.paths import SCRIPTS_DIR

APT_PACKAGES = {
    "postgres": "postgresql",
    "mariadb": "mariadb-server",
    "mysql": "mysql-server",
    "valkey": "valkey-server",
    "redis": "redis-server",
}


def provision(ctx):
    services = ctx.dbs.services
    if not services:
        return

    packages = [APT_PACKAGES[service] for service in services if service in APT_PACKAGES]
    if packages:
        apt.packages(
            name="Install selected database services",
            packages=packages,
            present=True,
            update=True,
            cache_time=3600,
            _sudo=True,
        )
    if "mongodb" in services:
        _install_mongodb()

    project = _database_identifier(ctx.app.project_name)
    env_path = f"{ctx.paths_dict['shared']}/.env"
    if "postgres" in services:
        _postgres(project, env_path)
    if "mariadb" in services:
        _mysql(project, env_path, "mariadb")
    if "mysql" in services:
        _mysql(project, env_path, "mysql")
    if "mongodb" in services:
        _mongodb(project, env_path)
    if "valkey" in services:
        _key_value_store(ctx, env_path, project, "valkey")
    if "redis" in services:
        _key_value_store(ctx, env_path, project, "redis")


def _database_identifier(project_name):
    name = project_name.replace("-", "_")
    if not name or len(name) > 48 or not name.replace("_", "").isalnum():
        raise ValueError("project_name cannot be used as a database identifier")
    return name


def _install_mongodb():
    server.script(
        name="Install MongoDB package source",
        src=str(SCRIPTS_DIR / "install-mongodb-repo.sh"),
        _sudo=True,
    )
    apt.packages(
        name="Install MongoDB",
        packages=["mongodb-org"],
        present=True,
        update=True,
        _sudo=True,
    )


def _postgres(project, env_path):
    user = f"{project}_postgres"
    server.script_template(
        name="Configure PostgreSQL for project",
        src=str(SCRIPTS_DIR / "configure-postgres-project.sh.j2"),
        env=env_path,
        user=user,
        project=project,
        _sudo=True,
    )
    systemd.service(
        name="Enable PostgreSQL", service="postgresql", enabled=True, running=True, restarted=True, _sudo=True
    )


def _mysql(project, env_path, implementation):
    user = f"{project}_mysql"
    server.script_template(
        name=f"Configure {implementation} for project",
        src=str(SCRIPTS_DIR / "configure-mysql-project.sh.j2"),
        env=env_path,
        user=user,
        project=project,
        _sudo=True,
    )
    systemd.service(
        name=f"Enable {implementation}", service="mysql", enabled=True, running=True, restarted=True, _sudo=True
    )


def _mongodb(project, env_path):
    user = f"{project}_mongodb"
    server.shell(
        name="Configure MongoDB for project",
        commands=[
            "sed -ri 's/^[[:space:]]*bindIp:.*/  bindIp: 127.0.0.1/' /etc/mongod.conf",
            "grep -q '^security:' /etc/mongod.conf || printf '\\nsecurity:\\n  authorization: enabled\\n' >> /etc/mongod.conf",
        ],
        _sudo=True,
    )
    systemd.service(name="Enable MongoDB", service="mongod", enabled=True, running=True, restarted=True, _sudo=True)
    admin_file = "/root/.config/bonesinfra/mongodb-admin.env"
    server.script_template(
        name="Create least-privilege MongoDB project user",
        src=str(SCRIPTS_DIR / "create-mongodb-project-user.sh.j2"),
        admin_file=admin_file,
        env=env_path,
        project=project,
        user=user,
        _sudo=True,
    )


def _key_value_store(ctx, env_path, project, service):
    unit, package_user, default_port = {
        "valkey": ("valkey-server", "valkey", 16379),
        "redis": ("redis-server", "redis", 16379),
    }[service]
    password_key = service.upper() + "_PASSWORD"
    port_key = service.upper() + "_PORT"
    url_key = service.upper() + "_URL"
    service_name = f"{project}-{service}"
    config = f"/etc/bonesinfra/dbs/{service_name}.conf"
    data_dir = f"/var/lib/{service}/{project}"
    unit_path = f"/etc/systemd/system/{service_name}.service"
    binary = f"/usr/bin/{package_user}-server"
    server.script_template(
        name=f"Configure isolated {service} instance for project",
        src=str(SCRIPTS_DIR / "setup-key-value-store.sh.j2"),
        env=env_path,
        config=config,
        data=data_dir,
        default_port=str(default_port),
        password_key=password_key,
        port_key=port_key,
        url_key=url_key,
        unit=unit,
        runtime_group=ctx.runtime.runtime_group,
        binary=binary,
        service=service,
        project=project,
        package_user=package_user,
        unit_path=unit_path,
        _sudo=True,
    )
    systemd.service(
        name=f"Enable isolated {service} instance",
        service=service_name,
        enabled=True,
        running=True,
        restarted=True,
        daemon_reload=True,
        _sudo=True,
    )
