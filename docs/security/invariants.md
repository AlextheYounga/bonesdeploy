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
2.  Every site build environment has a separate build UID.
3.  Runtime users have no login shell, password, or sudo rights.
4.  Runtime users are not members of cross-site supplementary groups.
5.  No shared Unix identity owns data belonging to multiple sites.
```

Three identities, not two and not five. The `git` user owns the bare repo and is ingress only. The `<site>` runtime user owns `shared/`, writable paths, and `/run/<site>` and mutates runtime state. `root` owns system units, config dirs, users, and sealed releases, and provisions, deploys, and restarts. The runtime user is dedicated per project — not `www-data`, not a shared `applications` user. One project, one user. Isolation is enforced by the kernel, not by your discipline.

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
```

Permissions are a provisioning-time contract, not a deployment-time repair. The ownership layout is established once during `bonesdeploy remote setup` and never rewritten by deploy commands. If you find yourself wanting to `chmod` during a deploy, you are fixing the wrong thing — fix the provisioning. `shared/` is owned by the runtime user; only the app writes there. `releases/` is owned by the runtime user while prepare runs, then sealed as `root:<site>` before activation. The setgid bit on `releases/` lets the runtime group inherit read access without a post-deploy `chown`.

No shared groups with `660`/`770` everywhere — that pattern is a tangle of logic traps. No ACLs — they're opaque and unreadable. Ordinary Unix ownership, every time.

## Privileged mediation

```text
13. BonesRemote accepts typed operations, not arbitrary root shell commands.
14. Site names and release IDs are validated before path construction.
15. All generated paths are constrained beneath canonical site roots.
16. Symlinks are rejected or safely resolved in privileged write operations.
17. Git hooks may trigger deployment but cannot perform deployment work.
18. User-controlled deployment input is never executed as root.
19. Runtime users cannot modify configuration later consumed as code by root.
```

BonesRemote is the privileged mediator. Its job is not to "run deployments" — its job is to constrain the deployer to a finite set of safe state transitions. It accepts narrow, typed operations like `activate_release(site="atlas", release="20260727_143200")`, never `run_as_root(command="...")` or `write_file(path="...", content="...")`.

Git hooks trigger BonesRemote. They do not check out source, run builds, write releases, or restart services. The `pre-push` guard runs `bonesdeploy doctor --local` and aborts on warnings. The remote `post-receive` trigger derives `<site>` from `GIT_DIR` and calls `sudo bonesremote hook post-receive --site <site>` — nothing more. The config repo's `pre-receive` trigger calls `bonesremote site receive` directly as root and atomically replaces control-plane state. The sudoers policy is rendered and validated by `bonesinfra` at provisioning time with anchored site and revision arguments, so trailing or malformed arguments are denied.

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
30. No application receives a container-engine socket.
31. No application receives BonesRemote's control socket.
32. Unix sockets have unique owners and restrictive modes.
33. Socket-activated descriptors are passed only intentionally.
34. Privileged APIs authenticate both the caller and requested site.
```

An open descriptor is already-granted authority. Permissions changed to `000` do not revoke an already-open file. Descriptors inherit across `execve()` unless marked close-on-exec and can be passed between processes via `SCM_RIGHTS`. This is why a Docker socket is so dangerous — it's not a file, it's a handle to a more privileged authority. BonesDeploy passes no engine socket and no control socket into applications, ever.

## Network

```text
35. Only the public reverse proxy binds public HTTP/HTTPS ports.
36. Application upstreams bind to loopback, private sockets, or private
    per-site networks.
37. Databases are not publicly published.
38. Shared databases use per-site databases and narrowly scoped accounts.
39. Internal reachability does not substitute for authentication.
40. Sites do not automatically share one unrestricted internal network.
```

Supported databases are PostgreSQL, MariaDB, MySQL, MongoDB, Valkey, and Redis. Every listener binds to localhost. Redis and Valkey use separate per-project instances; the SQL and Mongo services use database-scoped accounts. Generated credentials live in the protected remote `shared/.env`, never in `.bones/`. Remote workstation access uses ordinary SSH port forwarding; no tunnel information is stored. MariaDB and MySQL are mutually exclusive server implementations. Internal reachability is never a substitute for authentication — both must hold.

## Containers

```text
41. Container execution is rootless and associated with the site's UID.
42. Runtime definitions are generated by BonesDeploy, not accepted
    unrestricted from applications.
43. No privileged containers.
44. No host PID, host IPC, or host network namespace without exceptional need.
45. No arbitrary host bind mounts.
46. No engine socket inside containers.
47. Capabilities are dropped.
48. Persistent mounts are explicit and site-specific.
49. Image identity is recorded by immutable digest.
```

Containers still share a kernel trust boundary. Rootless execution reduces the consequence of many escapes because the outer host identity is unprivileged, but it does not remove the common kernel from the trusted computing base. BonesDeploy uses rootless Podman for isolated builds where the boundary is genuinely useful: the build runs inside a constrained environment, produces a release, and disappears. The build container gets the exported source tree and a private persistent build cache at `/workspace/cache`; it does **not** get `.env`, `shared/`, `current/`, `releases/`, the bare repo, or host `bonesremote` control-plane files. Build input is disposable. Build output is what gets promoted.

`bonesremote` runs each build script through the build user's systemd user manager with `systemd-run --machine=<site>-build@ --user`, not `runuser`. The long-lived build container is a transient user service that tracks Podman's monitor process; each script streams its output through foreground `podman exec`. Runtime definitions are generated by BonesDeploy — unrestricted user-provided Compose files are not accepted, because Compose fields are instructions to the daemon about mounts, namespaces, devices, capabilities, and networking.

## Availability

```text
50. Every runtime service has memory and task limits.
51. Every build has stricter memory, CPU, process, and time limits.
52. Logs and build caches have size bounds.
53. Databases have connection and role limits.
54. One site cannot consume every host port, inode, process, or byte of disk.
```

Resource exhaustion is a distinct security dimension. Good confidentiality does not imply good availability: site A may be unable to read site B and still allocate all host memory until site B is killed by system pressure. Every untrusted site and build has resource boundaries — `MemoryMax`, `TasksMax`, `CPUQuota` or `CPUWeight`, file-descriptor limits, build timeout, disk or filesystem quota, log-size policy. Beyond per-script timeouts, BonesInfra caps each build user's host-level slice at 80% CPU quota, 80% memory high/max, and `MemorySwapMax=0`, so a runaway build fails rather than exhausting host memory or swap.

## Just-in-time mutations

A mutation happens at the last responsible moment — immediately before the system would fail if it didn't. Not earlier. Not "while we're here."

- pre-deploy steps validate and prepare *isolated* state. They don't touch live state.
- build steps run on isolated workspace state.
- activation happens at activation time.
- permission hardening happens *after* a successful activation, not before.
- a failed deploy leaves no broadened access, no half-applied live mutations.

If a mutation can be delayed safely, it is delayed. If a mutation affects live state, it is justified by an immediate need. This is not aesthetic preference. It is the difference between a deploy that fails clean and a deploy that fails into a security incident.

## The lock

`bonesremote` holds one OS-backed deployment lock per site. Deploys, cancellations, and site imports all take it. Nothing stages or overwrites state while a release is building, preparing, or interrupted. The lock lives outside the replaceable site dataset, so replacing the dataset doesn't replace the lock. Before staging, BonesRemote starts and verifies the build user's systemd manager and checks rootless Podman readiness. A damaged rootless Podman namespace is reported before any release state is created — deploy does not silently reset Podman, because that operation stops the build user's containers.

## Service restart

`bonesremote service restart` restarts `<project>.target`, which restarts every registered site service. It is the only `bonesremote` command that needs root. `bonesinfra` owns site service membership. `bonesremote` restarts exactly `<project>.target` for deploy and rollback — nothing more, nothing less.

## Doctor: the fail-closed audit

`bonesdeploy doctor` is a read-only, fail-closed security audit. Required evidence that cannot be collected is reported as `UNVERIFIED` and causes doctor to fail rather than pass silently.

Site doctor verifies:

- site identity isolation — unique UIDs/GIDs, no login shells, no cross-site group membership, deploy not in runtime groups
- runtime sudo absence
- privileged configuration root-control — recursively inspecting systemd, sudoers, nginx, AppArmor, and BonesRemote state plus their parent chains without following symlink targets
- release activation — `current` must be a valid symlink resolving inside the site's `releases/` directory; active release roots and activation parents must be immutable to the runtime identity

`bonesremote doctor --site <project> --exhaustive` additionally inspects every entry in the active release for permission drift. It can take time on large releases. POSIX ACLs on protected paths are detected through extended attributes and reported as `UNVERIFIED`. Supplementary groups are collected through `id -G`.

Doctor reports three states: green checks are healthy, yellow pending items are expected next steps (such as the first Git push after setup), and red failures need attention. Pending first-push state exits successfully so setup can finish without looking broken. For agents and scripts, the stable machine-readable next-step guide is `bonesdeploy skill next --format json`.

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