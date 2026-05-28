# BonesDeploy Logging and Observability Policy

## Purpose

Logs should support incident response without leaking secrets or becoming writable by unrelated users.

## Logs Should Not Leak Secrets

Application logs should not contain:

- `.env` contents
- Authorization headers
- API tokens
- OAuth secrets
- database passwords
- private keys
- session cookies
- full request bodies containing credentials

## Log Permissions

Logs should be writable by the application or captured by journald, but not broadly writable by unrelated users.
Other service users should not be able to read sensitive logs.

## Audit Signals

The system should preserve logs useful for security review:

- systemd journal for application services
- auth logs
- sudo logs
- AppArmor denials
- web server access and error logs
- deployment logs

Missing or disabled logs for critical services should be treated as findings.

## Findings

The agent or operator should flag:

- logs containing secrets
- logs readable by unrelated service users
- critical audit logs missing or disabled
