Yes. Linux security becomes much easier to understand once you stop thinking in terms of “users,” “containers,” or “applications” and instead think in terms of **subjects holding authority over kernel objects**.

The kernel is essentially a reference monitor. A process asks the kernel to perform an operation—open a file, send a packet, signal another process, mount a filesystem—and the kernel decides whether the operation is permitted. Under the assumptions that the kernel, hardware, boot chain, and privileged control plane are not compromised, a denied operation is not merely discouraged: **the process has no mechanism by which to perform it**.

# The governing equation

For a process `P` to directly perform operation `O` on object `X`, the following must all be true:

```text
DIRECTLY_ALLOWED(P, O, X) =

    the process can identify or already holds X
AND the requested system call is permitted
AND its credentials or capabilities authorize O
AND its namespaces permit O in the relevant scope
AND every active security module permits O
AND object-specific rules permit O
AND resource limits permit O
```

But this is only half the security model. A process can also ask another process to act for it:

```text
EFFECTIVE_AUTHORITY(P) =

    P's direct kernel authority
    +
    every operation that a reachable service
    is willing to perform for P
```

That second equation explains Docker sockets, `sudo`, databases, deployment agents, HTTP APIs, D-Bus, SSH agents, and Unix sockets. A process does not need direct permission to modify `/etc`; it only needs access to a privileged service that will modify `/etc` on its behalf.

The central rule is therefore:

> **An action is impossible only when every direct and delegated path to that action is blocked.**

# The hard rules of Linux authority

## 1. A process can only use authority it possesses or receives

A Linux process’s security state includes:

```text
user and group IDs
supplementary groups
capabilities
namespace membership
LSM security context
seccomp filters
open file descriptors
mapped memory
accessible sockets
environment and secrets
cgroup placement
```

The kernel evaluates the process that actually makes the system call. It does not care that the source code belongs to “Site A” or that a process calls itself “nginx.” Names are administrative conventions; credentials and handles are what matter. Effective user IDs, group IDs, and supplementary groups are among the credentials used for access checks, and children normally inherit credentials from their parents. ([man7.org][1])

**Derived rule:** two services running under the same UID are not meaningfully isolated by ordinary filesystem permissions.

```text
site-a process: UID 1001
site-b process: UID 1001
```

To the discretionary access-control system, they are the same principal.

For BonesDeploy:

```text
one site = one unique runtime UID
one build environment = a separate build UID
```

This is a real security boundary, not merely organizational neatness.

---

## 2. Filesystem access is governed by every directory in the path

To open:

```text
/var/www/atlas/shared/.env
```

the process must be able to traverse every directory:

```text
/
└── var
    └── www
        └── atlas
            └── shared
                └── .env
```

For directories, the permission bits mean:

```text
read       list directory names
execute    traverse/search the directory
write      create, delete, or rename entries
```

For ordinary files, they mean:

```text
read       read file content
write      modify file content
execute    execute the file
```

Linux uses the owner, group, and other mode class, unless a relevant capability or another access-control mechanism changes the result. ([man7.org][2])

A subtle but extremely important consequence is:

> **Deleting a file is primarily an operation on its parent directory, not on the file.**

A process with write and search permission on a directory can ordinarily unlink or rename entries in that directory, even if the file itself belongs to root. The `unlink` permission check explicitly concerns write access to the containing directory and search permission through the path. ([man7.org][3])

Therefore, this is **not sealed**:

```text
drwxrwx--- atlas atlas /releases/123
-r--r----- root  atlas /releases/123/app.php
```

The `atlas` user may not be able to modify `app.php` in place, but because it controls the directory, it can potentially delete it and replace it with another file.

This is meaningfully sealed:

```text
drwxr-x--- root atlas /releases/123
-r--r----- root atlas /releases/123/app.php
```

The runtime user can read the release but cannot alter directory entries.

**Derived BonesDeploy invariant:**

```text
The runtime user must not own or write:

releases/
the active-release symlink's parent directory
systemd units
nginx configuration
deployment scripts
privileged hooks
BonesRemote state
```

It should own only intentionally mutable state.

---

## 3. Ownership itself is authority

A file owner can ordinarily change that file’s permission bits. Therefore, making a release `0444` is not a durable security boundary if the application user still owns it:

```text
-r--r--r-- atlas atlas app.php
```

The owner can potentially run:

```bash
chmod u+w app.php
```

and then modify it.

The stronger pattern is:

```text
-r--r----- root atlas app.php
```

Now the application can read the file but does not own it and cannot casually grant itself write permission. Capabilities such as `CAP_FOWNER`, `CAP_CHOWN`, and `CAP_DAC_OVERRIDE` can bypass various ownership and discretionary-access restrictions, which is why runtime processes should not retain them. ([man7.org][4])

This validates BonesDeploy’s existing candidate-to-sealed transition: the runtime user may prepare a candidate, but the promoted release becomes root-owned before activation.

---

## 4. An open file descriptor is already-granted authority

A pathname is one way to find an object. An open file descriptor is a direct reference to an object.

```text
pathname:
    /var/lib/secret

file descriptor:
    fd 7 → already-open secret file
```

Once `open()` succeeds, the resulting file descriptor refers to an open file description. Renaming or unlinking the pathname does not break that reference. ([man7.org][5])

This means:

```text
1. Process opens secret file.
2. Administrator changes permissions to 000.
3. Process still holds the previously opened descriptor.
4. Removing pathname access may not revoke that existing access.
```

File descriptors are also inherited across `execve()` unless they are marked close-on-exec, and they can be transferred between processes through Unix sockets using `SCM_RIGHTS`. ([man7.org][6])

Therefore:

> **Security is not determined only by what a process can open now. It also includes every handle it inherited or received earlier.**

Derived rules:

```text
Set O_CLOEXEC or FD_CLOEXEC by default.
Close unnecessary descriptors before launching applications.
Do not pass privileged sockets into untrusted services.
Audit systemd socket activation carefully.
Restart processes when attempting to revoke already-granted access.
```

This is why exposing a Docker socket is so dangerous. The socket is not merely a file; it is a handle to a more privileged authority.

---

## 5. Capabilities are explicit exceptions to normal rules

Linux divides traditional root authority into capabilities. Examples include:

```text
CAP_DAC_OVERRIDE       bypass many file permission checks
CAP_CHOWN              change file ownership
CAP_NET_ADMIN          administer networking
CAP_SYS_PTRACE         inspect or manipulate other processes
CAP_SYS_ADMIN          perform a very broad set of administrative operations
CAP_SYS_MODULE         load kernel modules
```

Capabilities are attached to threads and can be independently granted or dropped. ([man7.org][7])

Therefore, this reasoning is incomplete:

```text
The process is UID 1001, so it cannot modify that file.
```

The correct reasoning is:

```text
The process is UID 1001
AND is not in an authorized group
AND has no relevant capability
AND has no privileged file descriptor
AND cannot ask another service to modify it
AND the LSM permits no alternate route.
```

**Derived rule:**

```text
Runtime services should start with zero capabilities,
then receive only individually justified capabilities.
```

For a normal web application, the appropriate capability set is often empty.

---

## 6. Root inside a user namespace is not host root

User namespaces allow a process to appear as UID `0` inside a namespace while mapping to an ordinary unprivileged UID outside it:

```text
inside namespace:   UID 0
outside namespace:  UID 1007
```

Capabilities held in that user namespace generally apply only to resources governed by that namespace and descendant namespaces. User namespaces specifically isolate user/group IDs and capabilities. ([man7.org][8])

Therefore:

```text
container root
≠
host root
```

provided the container is genuinely user-namespaced or rootless.

But namespace root may still have considerable authority over:

```text
the container filesystem
processes in the container
objects mapped into the namespace
host files explicitly bind-mounted into it
services reachable from it
```

A user namespace limits where authority applies. It does not transform dangerous mounts or credentials into harmless ones.

---

## 7. Namespaces control visibility and scope, not complete authorization

Linux namespaces isolate particular categories of resources:

```text
mount namespace     visible filesystem mounts
network namespace   interfaces, routes, ports, firewall state
PID namespace       process-ID view
IPC namespace       certain shared-memory and message resources
UTS namespace       hostname and domain name
user namespace      IDs and namespace-scoped capabilities
```

Namespaces make resources appear as separate instances to processes inside them. ([man7.org][9])

However:

> **A namespace hides or scopes resources; it does not necessarily decide whether a visible resource may be used.**

For example:

```text
Mount namespace hides /home
    → /home cannot be reached by that pathname.

Bind mount /home/atlas into the namespace
    → it is visible again.

Pass an already-open descriptor for /home/atlas/secret
    → pathname visibility is irrelevant.
```

Similarly:

```text
Separate network namespace
    → separate interfaces and routes.

Add a route to a host database
    → database becomes reachable.

Database accepts supplied credentials
    → database operation becomes possible.
```

Namespaces are one gate among several, not the complete security policy.

---

## 8. Read-only views are only as strong as the absence of alternate writable paths

Suppose the same filesystem object is available through two paths:

```text
/read-only/app/config
/writable-host-mount/config
```

Making the first mount read-only does not make the object globally immutable if the process can reach the second path or already holds a writable descriptor.

Likewise, a process capable of changing its own mount setup may undo some mount-based restrictions. `CAP_SYS_ADMIN` includes many mount and namespace operations, which is one reason it is particularly dangerous. ([man7.org][7])

A valid proof of immutability therefore requires:

```text
no writable path to the object
no writable parent directory enabling replacement
no existing writable descriptor
no capability to remount or bypass restrictions
no privileged helper willing to modify it
```

Not merely:

```text
this particular path appears read-only
```

---

## 9. A reachable privileged service extends the caller’s authority

Consider an unprivileged process that cannot write `/etc/nginx`:

```text
application
    └── no direct write permission to /etc/nginx
```

Now add a root service:

```text
application
    │
    └── sends arbitrary pathname and content
            │
            ▼
root helper writes requested file
```

The application has effectively acquired a path to write arbitrary root-owned files. The kernel permissions remain intact; the root service is exercising its own authority on the application’s behalf.

This is the general rule behind:

```text
sudo
Docker sockets
deployment daemons
database servers
D-Bus system services
SSH agents
system management APIs
setuid programs
```

The security boundary becomes the privileged service’s authorization and input-validation logic.

**Derived rule:**

> A privileged mediator must accept narrow, typed operations—not arbitrary commands, paths, arguments, environment variables, or scripts.

Good:

```text
activate_release(site="atlas", release="20260727_143200")
```

Dangerous:

```text
run_as_root(command="...")
write_file(path="...", content="...")
restart_service(name_from_user_input)
```

BonesRemote is therefore one of BonesDeploy’s most security-sensitive components. Its responsibility is not merely to “run deployments.” It must constrain the deployer to a finite set of safe state transitions.

Your existing model reflects this: Git hooks trigger BonesRemote but do not themselves check out source, build releases, write live files, or restart services; BonesRemote mediates promotion and activation.

---

## 10. Writable configuration executed by a privileged process is privilege

Suppose an application cannot execute anything as root, but can modify:

```text
/etc/systemd/system/atlas.service
```

or:

```text
/root/.config/bonesremote/sites/atlas/deploy.sh
```

The next time root reloads or executes that configuration, application-controlled code runs with root authority.

Therefore:

```text
Can modify privileged code
=
can eventually exercise privileged authority
```

This includes:

```text
systemd units
sudoers files
cron definitions
shell hooks
nginx includes
environment files read by root
dynamic loader configuration
plugin directories
PATH directories used by root
executables launched by root
container definitions interpreted by a privileged daemon
```

**Derived rule:**

```text
Anything read as executable instructions by a privileged component
must be owned and writable only by a more trusted identity.
```

This is also why unrestricted user-provided Docker Compose files undermine a controlled deployment model. Compose fields are effectively instructions to the container daemon about mounts, namespaces, devices, capabilities, and networking.

---

## 11. `no_new_privs` closes the exec-based escalation path

Normally, executing another program can grant additional privilege through mechanisms such as:

```text
setuid binaries
setgid binaries
file capabilities
certain security-domain transitions
```

Once `no_new_privs` is set, `execve()` promises not to grant privileges the process did not already possess. The setting is inherited and cannot be unset. ([man7.org][10])

Thus:

```text
NoNewPrivileges=true
```

establishes a hard invariant:

```text
Executing a different binary cannot increase this process tree's privilege.
```

It does **not** mean:

```text
the process has no privilege
the process cannot use existing capabilities
the process cannot use an inherited privileged descriptor
the process cannot call a privileged service
```

It closes one class of escalation path; it does not erase existing authority.

---

## 12. Seccomp restricts system calls, not intentions

Seccomp filters system calls and, in some cases, their arguments. Filters are inherited by child processes and persist across `execve()`. ([man7.org][11])

For example:

```text
mount() blocked
ptrace() blocked
bpf() blocked
keyctl() blocked
```

Those direct operations become unavailable through those system calls.

But seccomp does not understand high-level intentions:

```text
“Do not alter production data.”
“Do not access another tenant.”
“Do not ask Docker to mount the host.”
```

If `write()` is allowed and the process already has a writable descriptor to a secret file, seccomp does not make that write safe. If networking is allowed and a management daemon accepts dangerous commands, seccomp does not understand the consequences of those messages.

Therefore:

> **Seccomp reduces the available kernel attack surface. It does not replace filesystem policy, network policy, authentication, or service authorization.**

---

## 13. LSM policy adds another independent denial layer

Linux Security Modules include systems such as AppArmor, SELinux, and Landlock. They add checks beyond ordinary Unix discretionary permissions. AppArmor tasks without a loaded profile run unconfined, which effectively leaves them subject only to the standard access-control mechanisms. ([Kernel.org][12])

Conceptually:

```text
DAC says allow
AND AppArmor says allow
    → operation may continue

DAC says allow
AND AppArmor says deny
    → operation denied

DAC says deny
AND AppArmor says allow
    → DAC still denies
```

Landlock is explicitly designed to add restrictions without granting access denied by the system’s other policies. ([Kernel.org][13])

This gives you a useful defense-in-depth property:

```text
Unix permissions accidentally too broad
    but AppArmor denies path
        → access remains denied
```

The reverse matters too:

```text
AppArmor profile accidentally absent
    but Unix permissions deny
        → access remains denied
```

Independent gates are valuable because one configuration mistake does not automatically destroy every boundary.

---

## 14. Network reachability and service authorization are separate gates

For an application to use a database, all of these must succeed:

```text
network route exists
firewall permits traffic
service is listening on that interface
application knows or can obtain credentials
database role permits the operation
```

Removing any one can make the operation impossible through that route.

For example:

```text
PostgreSQL listens only on 127.0.0.1
remote attacker has credentials
    → remote connection still impossible directly

Application can reach PostgreSQL
but role has access only to atlas database
    → access to lawsnipe database denied by PostgreSQL

Application cannot reach host database network
but has a local proxy socket that forwards queries
    → proxy becomes an alternate path
```

Network namespaces isolate network devices, routing, firewall state, and sockets, but any intentionally configured bridge, route, forwarded port, or proxy creates a path across that separation. ([man7.org][14])

BonesDeploy’s localhost-bound shared database model is therefore sound only when combined with database-scoped accounts and credentials. Your current design does both.

---

## 15. Secrets are transferable authority

A secret is not merely confidential data. It often represents permission:

```text
database password       → database authority
API token               → API authority
SSH private key         → remote-host authority
Docker socket           → container-daemon authority
cloud credential        → cloud-account authority
signing key              → release-signing authority
```

A process that can read a secret can often use it elsewhere, copy it, transmit it, or retain it after access is supposedly removed.

Therefore:

```text
Can read credential C
≈
can exercise whatever authority accepts C
```

unless additional restrictions such as network location, hardware-backed keys, short lifetimes, or audience binding prevent reuse.

This means application environment variables deserve the same scrutiny as filesystem permissions. A perfectly isolated application with administrator-level database credentials still has administrator-level database authority.

---

## 16. Resource exhaustion is a distinct security dimension

Filesystem and namespace isolation do not prevent a process from consuming:

```text
memory
CPU
processes and threads
file descriptors
disk space
I/O bandwidth
network connections
kernel objects
```

Cgroups constrain resource consumption; they do not primarily protect data confidentiality. The kernel specifically recommends memory cgroup limits for untrusted processes using user namespaces because namespace creation can otherwise expose resource-abuse risks. ([Kernel.org][15])

Thus, this system may have good confidentiality but poor availability:

```text
site A cannot read site B
site A can allocate all host memory
site B is killed by system-wide memory pressure
```

Derived rule:

```text
Every untrusted site and build must have resource boundaries.
```

At minimum:

```text
MemoryMax
TasksMax
CPUQuota or CPUWeight
file-descriptor limits
build timeout
disk or filesystem quota
log-size policy
```

Systemd exposes cgroup-backed memory, process, CPU, and I/O controls for services and user slices. ([FreeDesktop][16])

---

## 17. All containers still share a kernel trust boundary

Mount, PID, network, IPC, and user namespaces create strong barriers, but containerized processes still invoke the host kernel.

A normal compromised application container does not automatically control the host. Crossing from an appropriately confined container into host authority generally requires:

```text
dangerous pre-existing configuration
a privileged delegated service
a sensitive mounted object
an exposed runtime socket
or a kernel/runtime vulnerability
```

Rootless execution reduces the consequence of many escapes because the outer host identity is unprivileged. But it does not remove the common kernel from the trusted computing base.

So:

```text
container boundary
=
strong kernel-enforced isolation boundary

container boundary
≠
separate kernel boundary
```

A virtual machine introduces a different kernel boundary. A separate physical host introduces an additional hardware boundary.

# Concrete possible-versus-impossible examples

Assume no kernel vulnerability and no privileged service unexpectedly acting on the attacker’s behalf.

| Situation                                                                    | Result                                            | Reason                                               |
| ---------------------------------------------------------------------------- | ------------------------------------------------- | ---------------------------------------------------- |
| Site A tries to read Site B’s `0600` file owned by Site B                    | Impossible directly                               | Wrong UID, no group permission, no bypass capability |
| Site A knows the exact filename but cannot traverse Site B’s directory       | Impossible through pathname                       | Directory search permission fails                    |
| Site A previously opened the file, then permissions become `000`             | Potentially still possible                        | It retains the open descriptor                       |
| Site A cannot write a root-owned file but owns its parent directory          | Replacement may be possible                       | Directory authority can permit unlink/rename         |
| Site A reads a root-owned release inside a root-owned non-writable directory | Read possible, mutation directly impossible       | File is readable; file and parent are not writable   |
| Site A runs as UID 0 inside a rootless user namespace                        | Host-root actions ordinarily impossible           | Namespace UID 0 maps to an unprivileged host UID     |
| Site A has `CAP_DAC_OVERRIDE` in the host user namespace                     | Many DAC restrictions bypassable                  | Capability explicitly grants bypass authority        |
| Site A cannot write systemd units but can edit a script executed by root     | Root execution possible indirectly                | Writable privileged input is delegated authority     |
| Site A cannot access Docker’s socket                                         | It cannot directly command Docker                 | No path to daemon API                                |
| Site A can access the Docker socket                                          | Daemon-level operations become possible           | Docker becomes a privileged deputy                   |
| Seccomp blocks `mount()`                                                     | Direct `mount()` impossible                       | Kernel rejects the syscall                           |
| A privileged helper accepts “mount this path” requests                       | Mount potentially possible indirectly             | Helper invokes the blocked action                    |
| Site A has no route to Site B’s network                                      | Direct network connection impossible              | No network path                                      |
| Both sites share a database but have separate restricted roles               | Server reachable; cross-database operation denied | Database authorization remains another gate          |
| Site A has no cgroup memory limit                                            | Host-wide memory exhaustion may be possible       | No site-specific memory ceiling                      |
| Site A has a correctly enforced memory maximum                               | Consumption above that cgroup limit is prevented  | Kernel resource controller enforces the boundary     |

# The BonesDeploy security invariants

These are the rules I would make explicit and testable in BonesDeploy.

## Identity

```text
1. Every site has a unique runtime UID and GID.
2. Every site build environment has a separate build UID.
3. Runtime users have no login shell, password, or sudo rights.
4. Runtime users are not members of cross-site supplementary groups.
5. No shared Unix identity owns data belonging to multiple sites.
```

## Filesystem

```text
6. Root owns all release directories after preparation.
7. Runtime users cannot write the releases/ directory itself.
8. Runtime users cannot write the parent of the current symlink.
9. Runtime users own only declared shared paths and runtime directories.
10. Root owns systemd units, nginx configuration, deployment state,
    AppArmor profiles, and privileged scripts.
11. No root-executed PATH directory is writable by a runtime user.
12. Shared paths are an explicit allowlist, never framework-wide guesses.
```

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

## Handles and IPC

```text
29. Unneeded file descriptors are close-on-exec.
30. No application receives a container-engine socket.
31. No application receives BonesRemote's control socket.
32. Unix sockets have unique owners and restrictive modes.
33. Socket-activated descriptors are passed only intentionally.
34. Privileged APIs authenticate both the caller and requested site.
```

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

## Availability

```text
50. Every runtime service has memory and task limits.
51. Every build has stricter memory, CPU, process, and time limits.
52. Logs and build caches have size bounds.
53. Databases have connection and role limits.
54. One site cannot consume every host port, inode, process, or byte of disk.
```

# The security-proof checklist

For every protected object—another site’s files, Docker, the database, root configuration—ask these questions in order:

```text
1. Can the attacker see or name the object?
2. Does it already hold a descriptor or handle to it?
3. Can it inherit or receive such a handle?
4. Do its UID, GIDs, or file modes permit the operation?
5. Does it hold a capability that bypasses that denial?
6. Is the capability valid in the object's governing namespace?
7. Does a namespace expose or hide the object?
8. Does seccomp permit the required system call?
9. Does AppArmor, SELinux, or another LSM permit it?
10. Can the attacker reach a service that will perform the operation?
11. Can it modify code or configuration that a privileged process will
    later consume?
12. Does it possess a credential representing the same authority?
13. Can it exhaust a shared resource instead of accessing the object?
14. What happens if the application is fully attacker-controlled?
```

If every possible route is demonstrably blocked, then the operation is **impossible within the stated threat model**.

That final qualification is essential. Linux isolation can give you hard kernel-enforced impossibility against an ordinary compromised process. It cannot give you unconditional impossibility against a kernel exploit, compromised host root, malicious hardware, or an incorrectly trusted privileged mediator. The practical goal is to reduce the remaining trusted base to something small enough to understand and audit:

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

That is the clearest security model for BonesDeploy: **assume every deployed application is already compromised, then prove that its UID, handles, capabilities, namespaces, reachable services, and writable paths do not form a route to another site or the host control plane.**

[1]: https://www.man7.org/linux/man-pages/man7/credentials.7.html?utm_source=chatgpt.com "credentials(7) - Linux manual page"
[2]: https://www.man7.org/linux/man-pages/man7/path_resolution.7.html?utm_source=chatgpt.com "path_resolution(7) - Linux manual page"
[3]: https://www.man7.org/linux/man-pages/man2/unlinkat.2.html?utm_source=chatgpt.com "unlink(2) - Linux manual page"
[4]: https://www.man7.org/linux/man-pages/man7/capabilities.7.html?utm_source=chatgpt.com "capabilities(7) - Linux manual page"
[5]: https://www.man7.org/linux/man-pages/man2/open.2.html?utm_source=chatgpt.com "open(2) - Linux manual page"
[6]: https://man7.org/linux/man-pages/man2/execve.2.html?utm_source=chatgpt.com "execve(2) - Linux manual page"
[7]: https://man7.org/linux/man-pages/man7/capabilities.7.html?utm_source=chatgpt.com "capabilities(7) - Linux manual page"
[8]: https://man7.org/linux/man-pages/man7/user_namespaces.7.html?utm_source=chatgpt.com "user_namespaces(7) - Linux manual page"
[9]: https://man7.org/linux/man-pages/man7/namespaces.7.html?utm_source=chatgpt.com "namespaces(7) - Linux manual page"
[10]: https://man7.org/linux/man-pages/man1/setpriv.1.html?utm_source=chatgpt.com "setpriv(1) - Linux manual page"
[11]: https://man7.org/linux/man-pages/man2/seccomp.2.html?utm_source=chatgpt.com "seccomp(2) - Linux manual page"
[12]: https://www.kernel.org/doc/html/latest/userspace-api/lsm.html?utm_source=chatgpt.com "Linux Security Modules — The Linux Kernel documentation"
[13]: https://www.kernel.org/doc/html/latest/security/landlock.html?utm_source=chatgpt.com "Landlock LSM: kernel documentation — The Linux Kernel documentation"
[14]: https://www.man7.org/linux/man-pages/man7/network_namespaces.7.html?utm_source=chatgpt.com "network_namespaces(7) - Linux manual page"
[15]: https://www.kernel.org/doc/html/latest/admin-guide/namespaces/resource-control.html?utm_source=chatgpt.com "User namespaces and resource control — The Linux Kernel documentation"
[16]: https://www.freedesktop.org/software/systemd/man/250/homectl.html?utm_source=chatgpt.com "homectl"
