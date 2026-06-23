# BonesDeploy Web Server Policy

## Purpose

The web server should expose only the public surface of a project.
It should not provide a back door into release trees, shared state, deployment metadata, or sensitive files.

## Web Server User Separation

The web server should run as its own user such as:

```text
nginx
www-data
caddy
```

It should not run as a project `service_user` unless there is an explicit documented reason.

## Static File Access

The web server should read only the public or static directories needed for serving.
In the current BonesDeploy model, it should serve only from `web_root` under `current`.

Good:

```text
/var/www/app1 -> /srv/sites/app1/current
```

Bad:

```text
web root exposes /srv/sites/app1/current directly
web root exposes /srv/sites/app1
web root exposes /srv/sites
```

## Uploads

Uploaded files should not be executable.
Upload directories should avoid script execution.
For stacks such as PHP applications, upload directories must not allow uploaded scripts to execute.

## Findings

The agent or operator should flag:

- web roots that expose app root instead of `current/<web_root>`
- directory listing enabled unintentionally
- upload directories that allow script execution
- sensitive files accessible over HTTP such as `.env`, `.git`, backups, SQLite DBs, or logs
