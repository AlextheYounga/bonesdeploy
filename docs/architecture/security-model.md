# The Linux Authority Model

This is how BonesDeploy reasons about isolation. Read it once and the rest of the security story stops being magic.

The whole thing snaps into place the moment you stop thinking about "users," "containers," and "applications" and start thinking about **subjects holding authority over kernel objects**. The kernel is a reference monitor. A process asks it to open a file, send a packet, signal another process, mount a filesystem. The kernel says yes or no. Under the usual assumption that the kernel, hardware, boot chain, and privileged control plane are not compromised, a denied operation is not merely discouraged — **the process has no mechanism by which to perform it.** That is the entire game.

## The governing equation

For a process `P` to directly perform operation `O` on object `X`, every gate has to open at once:

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

That is half the model. The other half is delegation. A process can also ask another process to act for it:

```text
EFFECTIVE_AUTHORITY(P) =
    P's direct kernel authority
    +
    every operation that a reachable service
    is willing to perform for P
```

That second line is the one people forget. It explains Docker sockets, `sudo`, databases, deployment agents, HTTP APIs, D-Bus, SSH agents, and every Unix socket a privileged daemon ever opened. A process does not need direct permission to modify `/etc`. It only needs access to a privileged service willing to modify `/etc` on its behalf.

So the central rule, stated plainly:

> **An action is impossible only when every direct and delegated path to that action is blocked.**

Not "discouraged." Not "against policy." Blocked, by the kernel, at every gate. That is the standard we hold ourselves to.

## The hard rules

### 1. A process can only use authority it possesses or receives

A Linux process's security state is its credentials, not its name:

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

The kernel evaluates the process that actually makes the syscall. It does not care that your source code calls itself "Site A" or "nginx." Names are administrative convention; credentials and handles are what matter.

> **Derived rule:** two services running under the same UID are not meaningfully isolated by ordinary filesystem permissions. To the discretionary access-control system, they are the same principal.

For BonesDeploy: one site = one unique runtime UID. One build environment = a separate build UID. This is a real security boundary, not organizational neatness.

### 2. Filesystem access is governed by every directory in the path

To open `/var/www/atlas/shared/.env`, the process has to traverse every directory above it. Directory permission bits mean:

```text
read       list directory names
execute    traverse/search the directory
write      create, delete, or rename entries
```

The subtle, extremely important consequence:

> **Deleting a file is primarily an operation on its parent directory, not on the file.**

A process with write and search permission on a directory can ordinarily unlink or rename its entries, even files owned by root.

So this is **not sealed**:

```text
drwxrwx--- atlas atlas /releases/123
-r--r----- root  atlas /releases/123/app.php
```

`atlas` can't edit `app.php` in place, but because it owns the directory, it can delete it and replace it.

This is meaningfully sealed:

```text
drwxr-x--- root atlas /releases/123
-r--r----- root atlas /releases/123/app.php
```

The runtime user can read the release. It cannot alter directory entries.

> **Derived invariant:** the runtime user must not own or write `releases/`, the active-release symlink's parent directory, systemd units, nginx configuration, deployment scripts, privileged hooks, or BonesRemote state. It owns only intentionally mutable state.

### 3. Ownership itself is authority

A file owner can change permission bits. Making a release `0444` is not a durable boundary if the application user still owns it:

```text
-r--r--r-- atlas atlas app.php
```

The owner can `chmod u+w app.php` and modify it. The stronger pattern:

```text
-r--r----- root atlas app.php
```

The application can read it, but does not own it and cannot casually grant itself write. Capabilities like `CAP_FOWNER`, `CAP_CHOWN`, and `CAP_DAC_OVERRIDE` bypass these restrictions, which is why runtime processes should not retain them.

This is why BonesDeploy seals releases as `root:<site>` before activation. The runtime user prepares a candidate; root owns the release that goes live.

### 4. An open file descriptor is already-granted authority

A pathname is one way to find an object. An open file descriptor is a direct reference. Once `open()` succeeds, renaming or unlinking the pathname does not break that reference. Permissions changed to `000` do not revoke an already-open descriptor.

File descriptors are also inherited across `execve()` unless marked close-on-exec, and can be transferred between processes over Unix sockets via `SCM_RIGHTS`.

> **Security is not determined only by what a process can open now. It includes every handle it inherited or received earlier.**

Derived rules: set `O_CLOEXEC` by default. Close unnecessary descriptors before launching applications. Do not pass privileged sockets into untrusted services. Audit systemd socket activation. Restart processes when attempting to revoke already-granted access.

This is why exposing a Docker socket is so dangerous. The socket is not a file; it is a handle to a more privileged authority.

### 5. Capabilities are explicit exceptions to normal rules

Linux divides traditional root authority into capabilities: `CAP_DAC_OVERRIDE`, `CAP_CHOWN`, `CAP_NET_ADMIN`, `CAP_SYS_PTRACE`, `CAP_SYS_ADMIN`, `CAP_SYS_MODULE`, and so on. They are attached to threads and can be granted or dropped independently.

This reasoning is incomplete:

```text
The process is UID 1001, so it cannot modify that file.
```

The correct reasoning:

```text
The process is UID 1001
AND is not in an authorized group
AND has no relevant capability
AND has no privileged file descriptor
AND cannot ask another service to modify it
AND the LSM permits no alternate route.
```

> **Derived rule:** runtime services start with zero capabilities and receive only individually justified ones. For a normal web application, the appropriate capability set is usually empty.

### 6. Root inside a user namespace is not host root

User namespaces let a process appear as UID `0` inside a namespace while mapping to an ordinary unprivileged UID outside it. Capabilities held in that namespace generally apply only to resources governed by it.

```text
container root ≠ host root
```

...provided the container is genuinely user-namespaced or rootless. But namespace root still has considerable authority over the container filesystem, processes in the container, objects mapped into the namespace, host files explicitly bind-mounted in, and services reachable from it. A user namespace limits *where* authority applies. It does not transform dangerous mounts or credentials into harmless ones.

### 7. Namespaces control visibility and scope, not complete authorization

Linux namespaces isolate categories of resources: mount, network, PID, IPC, UTS, user. They make resources appear as separate instances to processes inside them.

> **A namespace hides or scopes resources; it does not necessarily decide whether a visible resource may be used.**

A mount namespace can hide `/home`, but bind-mount `/home/atlas` back in and it's visible again. Pass an open descriptor for `/home/atlas/secret` and pathname visibility is irrelevant. A separate network namespace gives you separate interfaces and routes, but add a route to a host database and supply credentials and the operation becomes possible.

Namespaces are one gate among several, not the complete security policy.

### 8. Read-only views are only as strong as the absence of alternate writable paths

If the same object is reachable through two paths, making one read-only does not make the object immutable. A process capable of changing its own mount setup can undo mount-based restrictions — `CAP_SYS_ADMIN` covers many mount and namespace operations, which is why it is particularly dangerous.

A real proof of immutability requires:

```text
no writable path to the object
no writable parent directory enabling replacement
no existing writable descriptor
no capability to remount or bypass restrictions
no privileged helper willing to modify it
```

Not merely "this particular path appears read-only."

### 9. A reachable privileged service extends the caller's authority

An unprivileged process that cannot write `/etc/nginx` suddenly can, if you add a root service that accepts an arbitrary pathname and content from it. The kernel permissions are intact; the root service is exercising its own authority on the application's behalf.

This is the general rule behind `sudo`, Docker sockets, deployment daemons, database servers, D-Bus system services, SSH agents, system management APIs, and setuid programs. The security boundary becomes the privileged service's authorization and input-validation logic.

> **Derived rule:** a privileged mediator must accept narrow, typed operations — not arbitrary commands, paths, arguments, environment variables, or scripts.

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

This is why BonesRemote is one of the most security-sensitive components in the system. Its job is not to accept arbitrary remote instructions. It constrains an explicit deployment request to a finite set of safe state transitions. There is no Git-hook deployment trigger; the local CLI requests deployment explicitly.

### 10. Writable configuration executed by a privileged process is privilege

If an application cannot execute anything as root but can modify `/etc/systemd/system/atlas.service` or `/root/.config/bonesremote/sites/atlas/deploy.sh`, the next time root reloads or executes that configuration, application-controlled code runs with root authority.

```text
Can modify privileged code = can eventually exercise privileged authority.
```

This covers systemd units, sudoers files, cron definitions, shell hooks, nginx includes, environment files read by root, dynamic loader configuration, plugin directories, `PATH` directories used by root, executables launched by root, and container definitions interpreted by a privileged daemon.

> **Derived rule:** anything read as executable instructions by a privileged component must be owned and writable only by a more trusted identity.

This is also why unrestricted user-provided Docker Compose files undermine a controlled deployment model. Compose fields are instructions to the daemon about mounts, namespaces, devices, capabilities, and networking. Runtime definitions are generated from project-local infrastructure instead.

### 11. `no_new_privs` closes the exec-based escalation path

Normally `execve()` can grant additional privilege through setuid binaries, setgid binaries, file capabilities, and certain security-domain transitions. Once `no_new_privs` is set, `execve()` promises not to grant privileges the process did not already possess. The setting is inherited and cannot be unset.

It establishes a hard invariant: executing a different binary cannot increase this process tree's privilege. It does **not** mean the process has no privilege, cannot use existing capabilities, cannot use an inherited privileged descriptor, or cannot call a privileged service. It closes one class of escalation path; it does not erase existing authority.

### 12. Seccomp restricts system calls, not intentions

Seccomp filters syscalls and, sometimes, their arguments. Filters are inherited by child processes and persist across `execve()`. Block `mount()`, `ptrace()`, `bpf()`, `keyctl()` and those direct operations become unavailable through those syscalls.

But seccomp does not understand high-level intentions. "Do not alter production data." "Do not access another tenant." "Do not ask Docker to mount the host." If `write()` is allowed and the process has a writable descriptor to a secret, seccomp does not make that write safe. If networking is allowed and a management daemon accepts dangerous commands, seccomp does not understand the consequences.

> **Seccomp reduces kernel attack surface. It does not replace filesystem policy, network policy, authentication, or service authorization.**

### 13. LSM policy adds another independent denial layer

Linux Security Modules — AppArmor, SELinux, Landlock — add checks beyond ordinary Unix discretionary permissions. AppArmor tasks without a loaded profile run unconfined.

```text
DAC says allow AND AppArmor says allow → operation may continue
DAC says allow AND AppArmor says deny → operation denied
DAC says deny  AND AppArmor says allow → DAC still denies
```

Landlock is explicitly designed to add restrictions without granting access denied by other policies. The defense-in-depth property is valuable: if Unix permissions are accidentally too broad but AppArmor denies the path, access is still denied. If the AppArmor profile is absent but Unix permissions deny, access is still denied. One configuration mistake does not automatically destroy every boundary. Independent gates are the point.

### 14. Network reachability and service authorization are separate gates

For an application to use a database, all of these must succeed:

```text
network route exists
firewall permits traffic
service is listening on that interface
application knows or can obtain credentials
database role permits the operation
```

Removing any one makes the operation impossible through that route. PostgreSQL listening only on `127.0.0.1` means a remote attacker with credentials still cannot connect directly. An application that can reach PostgreSQL but has a role scoped to the `atlas` database is denied access to the `lawsnipe` database by PostgreSQL itself.

BonesDeploy's localhost-bound shared database model is sound only when combined with database-scoped accounts and credentials. Both, not one.

### 15. Secrets are transferable authority

A secret is not merely confidential data. It represents permission:

```text
database password       → database authority
API token               → API authority
SSH private key         → remote-host authority
Docker socket           → container-daemon authority
cloud credential        → cloud-account authority
signing key             → release-signing authority
```

A process that can read a secret can often use it elsewhere, copy it, transmit it, or retain it after access is supposedly removed.

```text
Can read credential C ≈ can exercise whatever authority accepts C
```

...unless additional restrictions — network location, hardware-backed keys, short lifetimes, audience binding — prevent reuse. Application environment variables deserve the same scrutiny as filesystem permissions. A perfectly isolated application with administrator-level database credentials still has administrator-level database authority.

### 16. Resource exhaustion is a distinct security dimension

Filesystem and namespace isolation do not prevent a process from consuming memory, CPU, processes, threads, file descriptors, disk space, I/O bandwidth, network connections, or kernel objects. Cgroups constrain resource consumption; they do not primarily protect confidentiality.

You can have good confidentiality and poor availability:

```text
site A cannot read site B
site A can allocate all host memory
site B is killed by system-wide memory pressure
```

> **Derived rule:** every untrusted site and build has resource boundaries.** At minimum: `MemoryMax`, `TasksMax`, `CPUQuota` or `CPUWeight`, file-descriptor limits, build timeout, disk or filesystem quota, log-size policy. Systemd exposes cgroup-backed memory, process, CPU, and I/O controls for services and user slices.

### 17. All containers still share a kernel trust boundary

Mount, PID, network, IPC, and user namespaces create strong barriers, but containerized processes still invoke the host kernel. A compromised container does not automatically control the host. Crossing into host authority generally requires dangerous pre-existing configuration, a privileged delegated service, a sensitive mounted object, an exposed runtime socket, or a kernel/runtime vulnerability.

Rootless execution reduces the consequence of many escapes because the outer host identity is unprivileged. It does not remove the common kernel from the trusted computing base.

```text
container boundary  =  strong kernel-enforced isolation boundary
container boundary  ≠  separate kernel boundary
```

A virtual machine introduces a different kernel boundary. A separate physical host introduces an additional hardware boundary. Know which one you're standing inside.

## Possible versus impossible

Assume no kernel vulnerability and no privileged service unexpectedly acting on the attacker's behalf.

| Situation | Result | Reason |
| --- | --- | --- |
| Site A reads Site B's `0600` file owned by Site B | Impossible directly | Wrong UID, no group permission, no bypass capability |
| Site A knows the filename but cannot traverse Site B's directory | Impossible through pathname | Directory search permission fails |
| Site A previously opened the file, then permissions become `000` | Potentially still possible | Open descriptor retained |
| Site A cannot write a root-owned file but owns its parent directory | Replacement may be possible | Directory authority permits unlink/rename |
| Site A reads a root-owned release inside a root-owned non-writable directory | Read possible, mutation directly impossible | File readable; file and parent not writable |
| Site A runs as UID 0 inside a rootless user namespace | Host-root actions ordinarily impossible | Namespace UID 0 maps to unprivileged host UID |
| Site A has `CAP_DAC_OVERRIDE` in the host user namespace | Many DAC restrictions bypassable | Capability explicitly grants bypass authority |
| Site A cannot write systemd units but can edit a script executed by root | Root execution possible indirectly | Writable privileged input is delegated authority |
| Site A cannot access Docker's socket | Cannot directly command Docker | No path to daemon API |
| Site A can access the Docker socket | Daemon-level operations possible | Docker becomes a privileged deputy |
| Seccomp blocks `mount()` | Direct `mount()` impossible | Kernel rejects the syscall |
| A privileged helper accepts "mount this path" requests | Mount potentially possible indirectly | Helper invokes the blocked action |
| Site A has no route to Site B's network | Direct connection impossible | No network path |
| Both sites share a database but have separate restricted roles | Server reachable; cross-database operation denied | Database authorization is another gate |
| Site A has no cgroup memory limit | Host-wide memory exhaustion may be possible | No site-specific memory ceiling |
| Site A has a correctly enforced memory maximum | Consumption above the cgroup limit is prevented | Kernel resource controller enforces the boundary |

The companion doc, `docs/security/invariants.md`, turns these rules into the explicit, testable invariants BonesDeploy enforces and the checklist used to prove them.
