from pyinfra.operations import apt, server, systemd

from bonesinfra.config.paths import SCRIPTS_DIR
from bonesinfra.services.runtime.base import RuntimeService


class MongoDBService(RuntimeService):
    def provision(self, ctx):
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
        project = self._db_identifier(ctx.app.project_name)
        env_path = f"{ctx.paths_dict['shared']}/.env"
        server.shell(
            name="Configure MongoDB for project",
            commands=[
                "sed -ri 's/^[[:space:]]*bindIp:.*/  bindIp: 127.0.0.1/' /etc/mongod.conf",
                (
                    "grep -q '^security:' /etc/mongod.conf || "
                    "printf '\\nsecurity:\\n  authorization: enabled\\n' >> /etc/mongod.conf"
                ),
            ],
            _sudo=True,
        )
        systemd.service(
            name="Enable MongoDB",
            service="mongod",
            enabled=True,
            running=True,
            restarted=True,
            _sudo=True,
        )
        server.script_template(
            name="Create least-privilege MongoDB project user",
            src=str(SCRIPTS_DIR / "create-mongodb-project-user.sh.j2"),
            admin_file="/root/.config/bonesinfra/mongodb-admin.env",
            env=env_path,
            project=project,
            user=f"{project}_mongodb",
            _sudo=True,
        )


MONGODB_SERVICE = MongoDBService()
