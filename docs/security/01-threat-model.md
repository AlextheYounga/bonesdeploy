# BonesDeploy Threat Model

## Purpose

This document defines the target security posture for BonesDeploy-managed applications running on a single Linux server.
It is meant to be used by a human operator or automated agent when comparing the current host against the desired hardened baseline.

The goal is not perfect hostile multi-tenant isolation.
The goal is strong practical isolation between BonesDeploy projects controlled by the same operator, with reduced blast radius if one application, build process, dependency, web endpoint, or worker process is compromised.

For truly hostile workloads, this policy should be treated as an inner hardening layer only.
Hostile tenants should be isolated with VMs, microVMs, or separate hosts.

## BonesDeploy Model

BonesDeploy's isolation model depends on a few core identities and paths:

- `deploy_user` handles SSH and deployment orchestration.
- `service_user` runs the long-lived application process.
- `deploy_root` stores build state, release trees, shared state, and deploy-managed symlinks.
- `public_path` is the stable user-facing path, normally `/var/www/<project>`.

In the current model, `public_path` points to `deploy_root/current`, and `current` points to the active directory under `deploy_root/runtime/<release-id>`.

## Assumed Threats

The system should assume that any individual project may become compromised through:

- remote code execution in the web application
- malicious dependency installation
- compromised build script
- unsafe subprocess invocation
- uploaded file handling bugs
- template injection
- SSRF or unsafe network access
- credential leakage from readable config files, environment variables, or logs
- vulnerable language runtimes, frameworks, or package managers

The isolation design should limit what a compromised project can read, write, execute, connect to, consume, or escalate into.

## Primary Security Goals

A compromised project should not be able to:

- read another project's source code, secrets, uploads, caches, logs, or release trees
- modify another project's files
- read deployment SSH keys, backup credentials, cloud credentials, tokens, or root-owned configuration
- gain sudo or root access
- load kernel modules or mount filesystems
- change host network configuration
- inspect unrelated processes
- exhaust all host CPU, memory, PIDs, or disk IO without limits
- bind arbitrary privileged ports
- access internal services unless explicitly allowed
- mutate deployed code at runtime unless the behavior is explicitly designed and documented
- persist malicious changes outside the approved writable paths for that service

## BonesDeploy Expectations

Within BonesDeploy, good isolation means:

- deploy work happens just in time instead of mutating live state early
- build steps work in isolated staging state, not in the active public tree
- the active release becomes `service_user`-owned after activation and hardening
- `public_path` remains a stable symlink to the active release
- service processes see only the active release and approved writable paths
- failed deploys do not leave broadened permissions or partially mutated live state behind

## Findings

The agent or operator should flag:

- projects sharing one runtime identity
- a service that can read another project's release tree or secrets
- a service that can write outside its approved writable paths
- deployment credentials readable by runtime processes
- deploy flow that mutates live state earlier than necessary
