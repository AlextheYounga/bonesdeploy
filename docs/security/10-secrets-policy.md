# BonesDeploy Secrets Policy

## Purpose

Secrets should be stored only in project-specific protected locations and be readable only by the identities that actually need them.

## Secrets Placement

Secrets should be stored only in protected locations such as:

```text
/srv/sites/<project>/shared/.env
/etc/<project>/env
systemd EnvironmentFile with strict permissions
```

Secrets should not be stored in:

```text
Git repositories
release directories readable by unrelated users
world-readable files
web-served directories
shell history
shared build caches
shared temp directories
logs
```

## Secrets Access

Only the specific service or identity requiring a secret should be able to read it.
The deploy worker should not pass all global secrets to every build job.
Build jobs should receive only the minimum secrets required for that exact job.

## Findings

The agent or operator should flag:

- `.env` files world-readable
- `.env` files readable by unrelated service users
- secrets under public web roots
- secrets copied into release artifacts
- secrets present in logs
- secrets exposed through unit files readable by all users
- SSH private keys readable by service users
- package-manager tokens readable by untrusted build scripts
