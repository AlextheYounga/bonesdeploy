from shlex import quote

from pyinfra.operations import files, server

from bonesinfra.config.context import DEPLOY_USER
from bonesinfra.config.paths import ASSETS_DIR, SCRIPTS_DIR
from bonesinfra.pyinfra.operations import mkdir


def _user_env_command(user, command):
    q_user = quote(user)
    home = f"$(getent passwd {q_user} | cut -d: -f6)"
    return f"HOME={home} XDG_CONFIG_HOME={home}/.config {command}"


def setup_repo_and_project(ctx, paths):
    mkdir(
        name="Ensure bare repo parent directory exists",
        path=paths["repo_parent"],
        user=DEPLOY_USER,
        group=DEPLOY_USER,
    )

    server.shell(
        name="Initialize bare git repo",
        commands=[_user_env_command(DEPLOY_USER, f"git init --bare {quote(paths['repo'])}")],
        _sudo=True,
        _sudo_user=DEPLOY_USER,
    )

    repo = quote(paths["repo"])
    server.shell(
        name="Set bare repo default branch",
        commands=[f"git --git-dir {repo} symbolic-ref HEAD refs/heads/{ctx.app.deploy.branch}"],
        _sudo=True,
        _sudo_user=DEPLOY_USER,
    )

    files.put(
        name="Install bare repo post-receive hook",
        src=str(ASSETS_DIR / "hooks/post-receive"),
        dest=f"{paths['repo']}/hooks/post-receive",
        user=DEPLOY_USER,
        group=DEPLOY_USER,
        mode="0755",
        _sudo=True,
    )

    _setup_bones_repo(paths)

    mkdir(
        name="Ensure project root parent directory is traversable",
        path=paths["project_root_parent"],
        mode="0711",
    )

    mkdir(
        name="Ensure project root boundary exists",
        path=paths["project_root"],
        user="root",
        group="root",
        mode="0751",
    )

    mkdir(
        name="Ensure releases directory with setgid",
        path=paths["releases"],
        user="root",
        group=ctx.runtime.runtime_group,
        mode="2750",
    )

    mkdir(
        name="Ensure shared directory (owned by runtime user)",
        path=paths["shared"],
        user=ctx.runtime.runtime_user,
        group=ctx.runtime.runtime_group,
        mode="0750",
    )

    env_path = f"{paths['shared']}/.env"
    server.script_template(
        name="Seed blank .env in shared directory",
        src=str(SCRIPTS_DIR / "seed-blank-env.sh.j2"),
        env_path=env_path,
        runtime_user=ctx.runtime.runtime_user,
        runtime_group=ctx.runtime.runtime_group,
        _sudo=True,
    )

    mkdir(
        name="Ensure placeholder release directory exists",
        path=paths["placeholder_web_root"],
        user="root",
        group=ctx.runtime.runtime_group,
        mode="0750",
    )


def _setup_bones_repo(paths):
    bones_repo = quote(paths["bones_repo"])
    bones_repo_parent = quote(str(paths["bones_repo"].rsplit("/", 1)[0]))
    server.shell(
        name="Ensure root-owned .bones repository parent exists",
        commands=[f"mkdir -p {bones_repo_parent}"],
        _sudo=True,
    )
    server.shell(
        name="Initialize bare .bones repository",
        commands=[f"git init --bare {bones_repo}"],
        _sudo=True,
    )

    server.shell(
        name="Set .bones repo default branch to master",
        commands=[f"git --git-dir {bones_repo} symbolic-ref HEAD refs/heads/master"],
        _sudo=True,
    )

    server.shell(
        name="Remove legacy .bones repo post-receive hook",
        commands=[f"rm -f {bones_repo}/hooks/post-receive"],
        _sudo=True,
    )

    files.put(
        name="Install .bones repo pre-receive hook",
        src=str(ASSETS_DIR / "hooks/config-pre-receive"),
        dest=f"{paths['bones_repo']}/hooks/pre-receive",
        user="root",
        group="root",
        mode="0755",
        _sudo=True,
    )
