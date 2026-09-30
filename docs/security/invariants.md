# BonesDeploy Security Invariants

This is what we actually enforce. The companion doc, [`docs/architecture/security-model.md`](../architecture/security-model.md), explains the Linux authority model these invariants are built on. Read that first if any of the reasoning here feels hand-wavy.

The whole point of BonesDeploy is a small, auditable trusted computing base. We assume every deployed application is already compromised, and we prove that the application's UID, handles, capabilities, namespaces, reachable services, and writable paths do not form a route to another site or to the host control plane. If we can't prove it, we don't ship it.

## The trusted base, stated plainly

```text
trusted:
    kernel
    root provisioning
    BonesRemote's narrow state machine
    database authorization
    explicit security policy

untrusted:
    application code
    dependencies
    builds
    runtime users
    container images
    deployment repository contents
```

Everything below is what "trusted" actually commits to. Linux can give you hard, kernel-enforced impossibility against an ordinary compromised process. It cannot give you unconditional impossibility against a kernel exploit, compromised host root, malicious hardware, or an incorrectly trusted privileged mediator. The practical goal is to shrink the trusted base until it's small enough to understand and audit.

## Identity

```text
1.  Every site has a unique runtime UID and GID.
2.  Runtime users have no login shell, password, or sudo rights.
3.  Runtime users are not members of cross-site supplementary groups.
4.  No shared Unix identity owns data belonging to multiple sites.
```

Three identities, not two and not five. The `bonesdeploy` user is the deployment SSH entry point and artifact transport principal. The `<site>` runtime user owns `shared/`, writable paths, and `/run/<site>` and mutates runtime state. `root` owns system units, config dirs, deployment state, and sealed releases, and runs the allowlisted BonesRemote lifecycle. The runtime user is dedicated per project — not `www-data`, not a shared `applications` user. One project, one user. Isolation is enforced by the kernel, not by your discipline.

## Filesystem

```text
6.  Root owns all release directories after preparation.
7.  Runtime users cannot write the releases/ directory itself.
8.  Runtime users cannot write the parent of the current symlink.
9.  Runtime users own only declared shared paths and runtime directories.
10. Root owns systemd units, nginx configuration, deployment state,
    AppArmor profiles, and privileged scripts.
11. No root-executed PATH directory is writable by a runtime user.
12. Shared paths are an explicit allowlist, never framework-wide guesses.
13. Root owns the Borg repository and the per-site Borg passphrase file
    (mode 0600); runtime users can neither read the passphrase nor write
    backup state.
14. The Borg passphrase never appears in process arguments, cron files,
    logs, deployment descriptors, or the control-plane snapshot.
```

Permissions are a provisioning-time contract, not a deployment-time repair. The ownership layout is established by `bonesdeploy server setup` and site setup, and never rewritten by deploy commands. If you find yourself wanting to `chmod` during a deploy, you are fixing the wrong thing — fix the provisioning. `shared/` is owned by the runtime user; only the app writes there. The existing root-owned lifecycle creates, prepares, seals, and activates release candidates without granting `bonesdeploy` write authority over the release namespace.

No shared groups with `660`/`770` everywhere — that pattern is a tangle of logic traps. No ACLs — they're opaque and unreadable. Ordinary Unix ownership, every time.

## Privileged mediation

```text
13. The deploy identity may sudo only exact BonesRemote config-sync and deploy commands.
14. Site names and release IDs are validated before path construction.
15. All generated paths are constrained beneath canonical site roots.
16. Symlinks are rejected or safely resolved in privileged write operations.
17. Deployment is requested explicitly; no Git hook performs deployment work.
18. Repository-provided build and prepare scripts are never executed as root.
19. Runtime users cannot modify configuration later consumed as code by root.
```

BonesRemote owns the complete privileged deployment lifecycle. Sudoers permits
only the complete anchored argument forms `config sync --site <site>` and
`deploy --site <site>` for the root-owned binary. Arbitrary subcommands,
optional deployment arguments, reordered arguments, and trailing arguments are
denied. The policy requires sudo 1.9.10 or newer for argument regular-expression
support.

Deployment is requested explicitly with `bonesdeploy deploy`. The local
committed revision supplies the source and its `infra/deployment` scripts; the
server receives only the resulting artifact. There is no application Git hook,
production application repository, first-push workflow, or deploy-on-push
trigger. BonesInfra validates the sudoers policy at provisioning time; anchored
argument matching rejects trailing or malformed arguments.

## Process confinement

```text
20. Application services have no capabilities by default.
21. NoNewPrivileges is enabled.
22. Setuid and setgid transitions are disabled or made ineffective.
23. Unneeded namespace creation is blocked.
24. Device access is denied unless explicitly required.
25. Kernel interfaces such as modules, tunables, logs, and control groups
    are inaccessible to application services.
26. The host filesystem is read-only or invisible except for declared paths.
27. AppArmor is enforced, not merely installed.
28. Seccomp blocks unnecessary high-risk syscall families.
```

Runtime services run under systemd `ProtectSystem=strict`, `NoNewPrivileges=yes`, `PrivateTmp=yes`, and per-site AppArmor profiles. Per-project services run as the dedicated runtime user, not a shared `www-data`, so blast radius is bounded by the kernel — not by your hope. Capabilities start at zero and stay there unless a specific, justified one is needed. For a normal web application, the appropriate capability set is usually empty.

## Handles and IPC

```text
29. Unneeded file descriptors are close-on-exec.
30. BonesDeploy never automatically supplies a container-engine socket.
31. No application receives BonesRemote's control socket.
32. Unix sockets have unique owners and restrictive modes.
33. Socket-activated descriptors are passed only intentionally.
34. Privileged APIs authenticate both the caller and requested site.
```

An open descriptor is already-granted authority. Permissions changed to `000`
do not revoke an already-open file. Descriptors inherit across `execve()` unless
marked close-on-exec and can be passed between processes via `SCM_RIGHTS`. This
is why a Docker socket is so dangerous: it is a handle to a more privileged
authority. BonesDeploy passes no control socket and never automatically adds an
engine socket; a Compose project can explicitly request one as privileged input.

## Network

```text
35. Native application upstreams bind to loopback or private sockets.
36. Compose ingress uses loopback when managed nginx is configured.
37. Compose projects may publish ports directly in reduced-guarantee mode.
38. Supporting services are externally managed for native sites or declared by
    the project for Compose sites.
39. Internal reachability does not substitute for authentication.
40. Native sites do not automatically share one unrestricted internal network.
```

BonesDeploy does not provision built-in databases or caches. Native sites use
independently managed services. Compose sites define supporting services,
networks, credentials, and exposure in their project-owned Compose file.
`shared/.env` is available for Compose interpolation but enters a container only
when the Compose definition references it. Internal reachability is never a
substitute for authentication.

## Containers

```text
41. Native application builds run locally in unprivileged Docker containers.
42. Native runtime definitions are generated and protected by BonesDeploy.
43. Compose definitions are trusted privileged input with reduced guarantees.
44. BonesDeploy does not automatically mount an engine socket into containers.
45. Compose owns its images, users, mounts, namespaces, capabilities, and networks.
46. The Compose project identity is stable per site.
47. Compose named volumes survive deploy, rollback, and release pruning.
48. Release rollback does not roll back persistent data or external side effects.
```

Containers still share a kernel trust boundary. Local Docker execution does not remove the common kernel from the trusted computing base. BonesDeploy uses a local Docker build environment for native builds and Compose image builds: it receives the exported committed source tree, a scoped cache, fixed public metadata, and committed public `.env.build` values. It does **not** get the root `.env`, runtime secrets, `shared/`, `current/`, `releases/`, production repositories, host home, SSH agent, credential stores, or Docker socket. Build input is disposable. Build output is the artifact BonesRemote verifies before promotion. This is a compatibility and reproducibility boundary, not a complete hardened tenant sandbox.

For Compose sites, the conventional rootful Docker daemon executes the
project-owned Compose definition. BonesDeploy constrains its own control plane,
release paths, locking, and privileged entry points, but does not certify the
container settings chosen by that file. Directly published ports can bypass
managed nginx and firewall assumptions.

For native sites, BonesRemote verifies the artifact's site, local revision,
length, digest, paths, entry types, symlinks, file count, and extracted size
before promotion. It never runs application build scripts. The receiver caps
the manifest at 64 KiB, compressed payload at 2 GiB, entries at 100,000, paths
and symlink targets at 4 KiB, and expanded file content at 4 GiB. Compose image
inventories are capped at 128 services and are loaded from the artifact.

## Availability

```text
50. Every runtime service has memory and task limits.
51. Local builds have bounded cache and per-script time limits.
52. Databases have connection and role limits.
53. One site cannot consume every host port, inode, process, or byte of disk.
```

Resource exhaustion is a distinct security dimension. Good confidentiality does not imply good availability: site A may be unable to read site B and still allocate all host memory until site B is killed by system pressure. Runtime services have cgroup-backed memory, process, CPU, and I/O controls. Local builds use a scoped cache and per-script timeout; production never provides native build resources.

## Just-in-time mutations

A mutation happens at the last responsible moment — immediately before the system would fail if it didn't. Not earlier. Not "while we're here."

- pre-deploy steps validate and prepare *isolated* state. They don't touch live state.
- build steps run on isolated workspace state.
- activation happens at activation time.
- permission hardening happens *after* a successful activation, not before.
- a failed deploy leaves no broadened access, no half-applied live mutations.

If a mutation can be delayed safely, it is delayed. If a mutation affects live state, it is justified by an immediate need. This is not aesthetic preference. It is the difference between a deploy that fails clean and a deploy that fails into a security incident.

## The lock

`bonesremote` holds one OS-backed deployment lock per site. Deploys, cancellations, rollbacks, and recovery all take it. Nothing stages or overwrites state while a release is building, preparing, or interrupted. The lock lives outside replaceable site data. Before staging, BonesRemote verifies and safely receives the artifact; it does not create or run a native build environment.

## Service restart

Within the root-executed deployment lifecycle, BonesRemote restarts `<project>.target`, which restarts every registered site service. `bonesinfra` owns site service membership. BonesRemote restarts exactly `<project>.target` for deploy and rollback — nothing more, nothing less. The deploy identity does not receive a separate general service-management sudo grant.

## Doctor: the fail-closed audit

`bonesdeploy doctor` is a read-only, fail-closed security audit. Required evidence that cannot be collected is reported as `UNVERIFIED` and causes doctor to fail rather than pass silently.

Server doctor verifies the reusable baseline before any site is provisioned:

- Debian 12+ or Ubuntu 24.04+ on `x86_64`, and AppArmor support
- the global deploy identity and root-controlled BonesRemote roots
- the root-owned BonesRemote binary and valid global sudoers policy
- active UFW and fail2ban protection plus unattended-upgrades configuration
- the installed etckeeper executable that records `/etc` provisioning changes

Site doctor verifies:

- site identity isolation — unique UIDs/GIDs, no login shells, no cross-site group membership, deploy not in runtime groups
- runtime sudo absence
- privileged configuration root-control — recursively inspecting systemd, sudoers, nginx, AppArmor, and BonesRemote state plus their parent chains without following symlink targets
- release activation — `current` must be a valid symlink resolving inside the site's `releases/` directory; active release roots and activation parents must be immutable to the runtime identity

`bonesremote doctor --site <project> --exhaustive` additionally inspects every entry in the active release for permission drift. It can take time on large releases. POSIX ACLs on protected paths are detected through extended attributes and reported as `UNVERIFIED`. Supplementary groups are collected through `id -G`.

Doctor reports healthy, pending, and failed checks. Pending checks describe non-destructive next steps, not an implicit push or hook installation. For agents and scripts, the stable machine-readable next-step guide is `bonesdeploy skill next --format json`.

## The security-proof checklist

For every protected object — another site's files, Docker, the database, root configuration — ask these questions in order:

```text
1.  Can the attacker see or name the object?
2.  Does it already hold a descriptor or handle to it?
3.  Can it inherit or receive such a handle?
4.  Do its UID, GIDs, or file modes permit the operation?
5.  Does it hold a capability that bypasses that denial?
6.  Is the capability valid in the object's governing namespace?
7.  Does a namespace expose or hide the object?
8.  Does seccomp permit the required system call?
9.  Does AppArmor, SELinux, or another LSM permit it?
10. Can the attacker reach a service that will perform the operation?
11. Can it modify code or configuration that a privileged process will
    later consume?
12. Does it possess a credential representing the same authority?
13. Can it exhaust a shared resource instead of accessing the object?
14. What happens if the application is fully attacker-controlled?
```

If every possible route is demonstrably blocked, the operation is **impossible within the stated threat model**. That final qualification is essential. The goal is not a magic spell that says "secure." The goal is a small enough trusted base that you can actually look at it and tell.
