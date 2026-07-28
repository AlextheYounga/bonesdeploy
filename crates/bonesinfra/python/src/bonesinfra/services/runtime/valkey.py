from pyinfra.operations import apt, server, systemd

from bonesinfra.config.paths import SCRIPTS_DIR
from bonesinfra.services.runtime.base import RuntimeService


class ValKeyService(RuntimeService):
    service = "valkey"
    unit = "valkey-server"
    package_user = "valkey"
    default_port = 16379

    def provision(self, ctx):
        apt.packages(
            name=f"Install {self.unit}",
            packages=[self.unit],
            present=True,
            update=True,
            cache_time=3600,
            _sudo=True,
        )
        project = self._db_identifier(ctx.app.project_name)
        env_path = f"{ctx.paths_dict['shared']}/.env"
        service_name = f"{project}-{self.service}"
        server.script_template(
            name=f"Configure isolated {self.service} instance for project",
            src=str(SCRIPTS_DIR / "setup-key-value-store.sh.j2"),
            env=env_path,
            config=f"/etc/bonesinfra/dbs/{service_name}.conf",
            data=f"/var/lib/{self.service}/{project}",
            default_port=str(self.default_port),
            password_key=self.service.upper() + "_PASSWORD",
            port_key=self.service.upper() + "_PORT",
            url_key=self.service.upper() + "_URL",
            unit=self.unit,
            runtime_group=ctx.runtime.runtime_group,
            binary=f"/usr/bin/{self.package_user}-server",
            service=self.service,
            project=project,
            package_user=self.package_user,
            unit_path=f"/etc/systemd/system/{service_name}.service",
            _sudo=True,
        )
        systemd.service(
            name=f"Enable isolated {self.service} instance",
            service=service_name,
            enabled=True,
            running=True,
            restarted=True,
            daemon_reload=True,
            _sudo=True,
        )


VALKEY_SERVICE = ValKeyService()
