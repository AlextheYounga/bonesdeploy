# BonesDeploy Logging and Observability Policy

## Purpose

Logs should help with incident response without leaking secrets or becoming writable by unrelated users.

## Rules

- Logs must not contain secrets, tokens, or private key material.
- Logs should be owned and permissioned so unrelated service users cannot read them.
- Keep systemd journal, auth logs, sudo logs, AppArmor denials, web logs, and deployment logs available for review.

## BonesDeploy Notes

- Post-deploy and service restart flows should keep log visibility intact.
- If logs live under a project directory, that directory should remain out of the web server path.

## Findings

- logs contain secrets
- logs are readable by unrelated service users
- critical audit logs are missing or disabled
