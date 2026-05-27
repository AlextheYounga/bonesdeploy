# BonesDeploy Web Server Policy

## Purpose

The web server should expose only the public surface of a project.
It should never become a back door into release trees, shared state, or deployment metadata.

## Rules

- Web server users should stay separate from service users.
- Static roots should point only at `public_path`, which resolves to `/var/www/<project>` and then to `/srv/deployments/<project>/current`.
- Upload directories should not execute code.
- Sensitive files such as `.env`, `.git`, backups, and databases must not be web-accessible.

## BonesDeploy Notes

- `docs/commands/bonesremote/landlock-nginx.md` should serve from `public_path` and not the whole release tree.
- `docs/commands/bonesremote/release-activate.md` makes the served path follow the active release atomically.
- `docs/commands/bonesremote/hooks-post-deploy.md` should harden ownership after activation, not before.

## Findings

- web root exposes the app root instead of `public_path`
- directory listing enabled unintentionally
- upload directory can execute scripts
- `.env`, `.git`, logs, or databases are reachable over HTTP
