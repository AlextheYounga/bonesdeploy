# BonesDeploy SSH Policy

## Purpose

SSH should be key-based, restricted, and kept separate from runtime service identities.

## SSH Access

SSH should use key-based authentication.
Recommended baseline:

```text
PasswordAuthentication no
PermitRootLogin no
PubkeyAuthentication yes
```

Root login should be disabled unless there is a documented emergency access model.

## SSH Keys

Deployment SSH keys should be readable only by the deploy user or root.
Service users should not be able to read deployment keys.

## Findings

The agent or operator should flag:

- private keys world-readable or group-readable by broad groups
- service users with SSH private keys
- shared SSH keys across unrelated projects
- root login enabled without justification
- password auth enabled on internet-facing SSH without justification
