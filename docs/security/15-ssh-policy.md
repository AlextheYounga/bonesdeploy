# BonesDeploy SSH Policy

## Purpose

SSH should be key-based, restricted, and separated from runtime service identities.

## Rules

- Password auth should be disabled on exposed SSH unless there is a specific reason.
- Root login should be disabled unless there is a documented emergency path.
- Deployment SSH keys should be readable only by the deploy identity or root.
- Service users should not read deployment keys.

## BonesDeploy Notes

- `deploy_user` is the SSH/deployment identity, not the service runtime identity.
- `bonesdeploy init` should keep the deploy and service identities distinct.

## Findings

- password auth enabled on internet-facing SSH
- root login enabled without justification
- private keys are group-readable by broad groups
- service users can read deployment keys
