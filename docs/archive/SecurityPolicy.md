# BonesDeploy Linux App Isolation Security Policy

## Purpose

This document defines the target security posture for BonesDeploy-managed applications running on a single Linux server. It is intended to be used by a human operator or an automated agent to compare the current server configuration against a desired hardened baseline.

The goal is not perfect hostile multi-tenant isolation. The goal is strong practical isolation between BonesDeploy projects controlled by the same operator, with reduced blast radius if one application, build process, dependency, web endpoint, or worker process is compromised.

For truly hostile multi-tenant workloads, this policy should be treated as an inner hardening layer only. Hostile tenants should be isolated with VMs, microVMs, or separate servers.

---

# 1. Threat Model

## 1.1 Assumed Threats

The system should assume that any individual app may become compromised through:

- Remote code execution in the web application
- Malicious dependency installation
- Compromised build script
- Unsafe subprocess invocation
- Uploaded file handling bug
- Template injection
- SSRF or unsafe network access
- Credential leakage from environment variables or readable config files
- Vulnerable language runtime, framework, or package manager

The isolation design should limit what a compromised app can read, write, execute, connect to, consume, or escalate into.

## 1.2 Primary Security Goals

A compromised app should not be able to:

- Read another app's source code, secrets, uploads, SQLite database, cache, logs, or release directories
- Modify another app's files
- Read server-wide secrets such as SSH keys, cloud credentials, deployment tokens, backup credentials, or root-owned config
- Gain sudo/root access
- Load kernel modules
- Mount filesystems
- Change network configuration
- Trace or inspect unrelated processes
- Exhaust all server CPU, memory, process IDs, or disk IO
- Bind arbitrary privileged ports
- Access internal services unless explicitly allowed
- Mutate its deployed code directory at runtime unless designed to do so
- Persist malicious changes outside approved writable directories

## 1.3 Non-Goals

This policy does not claim to fully protect against:

- Linux kernel privilege escalation bugs
- Physical access to the server
- A malicious root user
- A malicious hosting provider
- All side-channel attacks
- All denial-of-service attacks
- Fully hostile commercial multi-tenancy without VM or microVM boundaries

---

# 2. Filesystem Layout Policy

## 2.1 Preferred App Root

BonesDeploy projects should be deployed under:

```text
/srv/apps/<app-name>/
```

Preferred BonesDeploy app structure:

```text
/srv/apps/<app-name>/
  current -> releases/<release-id>
  releases/
    <release-id>/
  shared/
    .env
    storage/
    uploads/
  cache/
  tmp/
  repo/
  logs/        # optional; /var/log/<app-name> is also acceptable
```

Only the app's `public_path` should be exposed by the web server, for example:

```text
/srv/apps/<app-name>/current/public
```

The web server must not expose:

```text
/srv/apps/<app-name>/current
/srv/apps/<app-name>/shared
/srv/apps/<app-name>/releases
/srv/apps/<app-name>/repo
/srv/apps/<app-name>/tmp
/srv/apps/<app-name>/cache
```

## 2.2 `/srv` vs `/var/www`

`/srv/apps` is preferred over `/var/www` for BonesDeploy-managed applications because it represents service/application data, not merely web document roots.

`/var/www` is acceptable only for simple static sites or conventional web roots where no private source code, secrets, runtime state, or deployment metadata is stored beneath the served path.

## 2.3 Ownership Rules

Each app must have its own Unix service user and group:

```text
app1 -> user app1, group app1
app2 -> user app2, group app2
```

Expected ownership and control:

```text
/srv/apps/app1              root:root or deploy:deploy
/srv/apps/app1/releases     deploy:deploy while staging
/srv/apps/app1/current      symlink managed by deploy/root
/srv/apps/app1/shared       service-user:service-user or deploy:service-user with strict modes
/srv/apps/app1/cache        service-user:service-user
/srv/apps/app1/tmp          service-user:service-user
/srv/apps/app1/logs         service-user:service-user or root:adm depending on logging model
```

The deploy user prepares releases, but the active release tree should be service-user-owned after activation and post-deploy hardening. A good default is:

```text
staging/build workspace: deploy-owned or root-owned
active release tree: service-owned
runtime writable dirs: service-owned
```

## 2.4 Permission Rules

Default permissions should prevent cross-app reads and writes.

Suggested baseline:

```text
/srv/apps                  0751 root:root
/srv/apps/<app>            0750 root:<service-group> or deploy:<service-group>
/srv/apps/<app>/releases   0750 deploy:<service-group>
/srv/apps/<app>/shared     0750 <service-user>:<service-group>
/srv/apps/<app>/tmp        0700 <service-user>:<service-group>
/srv/apps/<app>/cache      0700 <service-user>:<service-group>
```

Secret files should be stricter:

```text
.env                       0640 root:<service-group> or deploy:<service-group>
private keys               0600 owner-only
SQLite DBs                 0600 or 0640 depending on group access
```

World-readable app directories should be treated as a finding unless explicitly justified.

## 2.5 Writable Directory Rules

Service processes should only be able to write to explicitly approved directories, such as:

```text
/srv/apps/<app>/shared/storage
/srv/apps/<app>/shared/uploads
/srv/apps/<app>/cache
/srv/apps/<app>/tmp
/run/<app>
/var/log/<app> or /srv/apps/<app>/logs
```

Service users should not be able to write to:

```text
/srv/apps/<app>/current
/srv/apps/<app>/releases
/usr
/etc
/bin
/sbin
/lib
/lib64
/root
/home
/boot
```

---

# 3. Identity and User Policy

## 3.1 One App, One Unix User

Each app should run as a dedicated unprivileged Unix service user.

Bad:

```text
all apps run as www-data
all apps run as deploy
all apps run as root
```

Good:

```text
app1 runs as app1
app2 runs as app2
app3 runs as app3
```

## 3.2 No Login Shells for Service Users

Service users should generally have no home directory and no interactive shell:

```text
/usr/sbin/nologin
/bin/false
```

Exception: temporary debugging access may be granted, but it should be time-limited and removed afterward.

## 3.3 No Sudo for Service Users

Service users must not have sudo privileges.

The following should be treated as critical findings:

```text
service user in sudo group
service user in wheel group
service user has NOPASSWD sudo rule
service user can run package manager commands with sudo
service user can restart arbitrary system services with sudo
```

## 3.4 Deploy User Separation

The deployment user should be distinct from service users.

Example:

```text
deploy user: deploy or bonesdeploy
service user: app1, app2, app3
web server user: nginx or www-data
```

The deploy user may manage releases and symlinks, but service users should not be able to mutate deployment metadata or other app releases.

---

# 4. systemd Service Hardening Policy

Each long-running app should be managed by a dedicated systemd service unless there is a specific reason not to.

## 4.1 Required or Strongly Preferred Settings

Each app service should use as many of the following as practical:

```ini
[Service]
User=<service-user>
Group=<service-group>
WorkingDirectory=/srv/apps/<app>/current
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/srv/apps/<app>/shared /srv/apps/<app>/cache /srv/apps/<app>/tmp /run/<app>
CapabilityBoundingSet=
AmbientCapabilities=
RestrictSUIDSGID=true
LockPersonality=true
MemoryDenyWriteExecute=true
PrivateDevices=true
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectKernelLogs=true
ProtectControlGroups=true
RestrictRealtime=true
SystemCallArchitectures=native
TasksMax=256
MemoryMax=<appropriate-limit>
CPUQuota=<appropriate-limit>
```

Some apps may require exceptions. Exceptions should be explicit and documented.

## 4.2 `ProtectSystem=strict`

`ProtectSystem=strict` should be preferred where possible. It makes most of the filesystem read-only to the service, with writable paths explicitly re-opened through `ReadWritePaths`.

If `ProtectSystem=strict` breaks an app, the agent should identify exactly which path required write access and recommend adding the narrowest possible `ReadWritePaths` entry instead of disabling the protection globally.

## 4.3 `ProtectHome=true`

Services should not normally read `/home`, `/root`, or user home directories.

If an app needs home-directory access, that should be treated as suspicious unless explicitly justified.

## 4.4 `NoNewPrivileges=true`

Services should use:

```ini
NoNewPrivileges=true
```

This prevents gaining new privileges through exec transitions such as setuid binaries.

## 4.5 Capabilities Policy

Most app services should have no Linux capabilities:

```ini
CapabilityBoundingSet=
AmbientCapabilities=
```

If a service needs a capability, the specific capability should be documented.

High-risk capabilities that should almost never be granted to app services:

```text
CAP_SYS_ADMIN
CAP_NET_ADMIN
CAP_SYS_MODULE
CAP_SYS_PTRACE
CAP_DAC_OVERRIDE
CAP_DAC_READ_SEARCH
CAP_SETUID
CAP_SETGID
CAP_CHOWN
CAP_FOWNER
CAP_SYS_RAWIO
CAP_MKNOD
```

## 4.6 Resource Limits

Each app should have cgroup-backed resource limits through systemd.

Recommended controls:

```ini
MemoryMax=
MemoryHigh=
CPUQuota=
TasksMax=
IOWeight=
```

The exact values depend on the app, but unlimited memory and unlimited task creation should be treated as findings.

---

# 5. AppArmor Policy

## 5.1 AppArmor Should Be Enabled

AppArmor should be installed, enabled, and enforcing where supported by the distribution.

The agent should check:

```bash
aa-status
systemctl status apparmor
cat /sys/module/apparmor/parameters/enabled
```

Expected result:

```text
AppArmor enabled
profiles loaded
target services in enforce mode
```

## 5.2 Per-App Profiles

Each long-running app service should have either:

- A dedicated AppArmor profile, or
- A profile generated by the container/runtime system, or
- A documented reason why AppArmor is not used for that service

A dedicated profile should restrict:

- Readable paths
- Writable paths
- Executable paths
- Network permissions where practical
- Access to sensitive `/proc` and `/sys` paths where practical
- Cross-app access

## 5.3 Minimum AppArmor Intent

For app `app1`, the effective AppArmor policy should approximate:

```text
allow read:       /srv/apps/app1/current/**
allow read:       /srv/apps/app1/shared/.env
allow read/write: /srv/apps/app1/shared/storage/**
allow read/write: /srv/apps/app1/cache/**
allow read/write: /srv/apps/app1/tmp/**
allow read/write: /run/app1/**
deny:             /srv/apps/app2/**
deny:             /srv/apps/app3/**
deny:             /root/**
deny:             /home/**
deny:             /etc/ssh/**
deny:             /var/lib/private/**
```

## 5.4 AppArmor Findings

The agent should flag:

- App services running unconfined
- Profiles in complain mode instead of enforce mode
- Broad read access to `/srv/apps/**`
- Broad read access to `/home/**` or `/root/**`
- Broad write access outside approved app directories
- Permission to execute arbitrary writable files
- Permission to read all of `/etc/**` without need
- Permission to access SSH private keys
- Permission to access other apps' `.env`, databases, uploads, or storage

---

# 6. Landlock Policy

## 6.1 Landlock Usage Model

Landlock should be used where the application or deployment worker can voluntarily reduce the permissions of child processes before executing risky code.

Good Landlock use cases:

- Build jobs
- User-submitted code
- Plugin execution
- Dependency install steps
- Asset compilation
- Project-specific deployment hooks
- Any child process that should only access a small workspace

Landlock is not a full replacement for AppArmor. AppArmor is the system-enforced static policy layer. Landlock is the dynamic per-process or per-job sandboxing layer.

## 6.2 Deployment Worker Landlock Policy

A deployment worker may have broad enough access to manage app deployments, but each child job should be restricted to only the paths it needs.

For job `job-123` deploying `app1`, the Landlock policy should approximate:

```text
allow read:       /usr
allow read:       /bin
allow read:       /lib
allow read:       /lib64
allow read:       /etc/ssl
allow read:       /srv/apps/app1/current
allow read/write: /srv/apps/app1/tmp/job-123
allow read/write: /srv/apps/app1/cache
allow read/write: /srv/apps/app1/releases/<new-release>
deny:             /srv/apps/app2
deny:             /srv/apps/app3
deny:             /srv/apps/app1/shared/.env unless required
deny:             /root
deny:             /home
deny:             /etc/ssh
```

If the job does not need network access, network access should be denied or avoided through additional controls.

## 6.3 Landlock Findings

The agent should flag:

- Risky child jobs run without any sandbox
- Build scripts inherit access to deployment secrets unnecessarily
- Jobs can read all app directories
- Jobs can write outside their workspace
- Jobs can access SSH keys, `.env` files, database files, or tokens not needed for the task
- Landlock is assumed to protect a process that never actually invokes it

---

# 7. cgroups and Resource Isolation Policy

## 7.1 Resource Limits Required

Every app service should have resource limits appropriate to its role.

Minimum recommended controls:

```ini
TasksMax=256
MemoryMax=<app-specific>
CPUQuota=<app-specific>
```

For heavier apps, use documented higher values.

## 7.2 Goals

cgroups should prevent:

- Fork bombs
- Unlimited memory growth
- One app monopolizing CPU
- Excessive process/thread creation
- Some forms of IO abuse

## 7.3 Findings

The agent should flag:

- No `TasksMax` or very high task limits
- No memory limit on app services
- No CPU control for untrusted or bursty workers
- Build workers with no cgroup limits
- Multiple apps sharing one service cgroup unnecessarily

---

# 8. seccomp Policy

## 8.1 seccomp Recommended

Where practical, services should use seccomp syscall filtering.

For systemd services, consider:

```ini
SystemCallFilter=
SystemCallArchitectures=native
```

For containers, use runtime seccomp profiles.

## 8.2 High-Risk Syscalls

The agent should identify whether app services have access to high-risk syscall families such as:

- Mounting filesystems
- Kernel module operations
- Raw IO
- ptrace
- keyring abuse
- namespace creation when not required
- BPF operations when not required

Systemd hardening options may cover some of these more safely than writing a custom syscall list manually.

## 8.3 Findings

The agent should flag:

- Services with no syscall restrictions when they run untrusted code
- Broad permission for namespace creation in untrusted workers
- Broad permission for mount-related syscalls
- `ptrace` available without need
- BPF-related permissions available without need

---

# 9. Network Isolation Policy

## 9.1 Inbound Network Exposure

Only necessary public ports should be exposed.

Typical public ports:

```text
22/tcp   SSH, ideally restricted by source IP or key-only auth
80/tcp   HTTP redirect / ACME challenge
443/tcp  HTTPS
```

Application backend ports should bind to localhost or private interfaces only.

Bad:

```text
app listens publicly on 3000, 8000, 8080, 9000
```

Good:

```text
nginx listens on 80/443
app listens on 127.0.0.1:3000 or Unix socket
```

## 9.2 Reverse Proxy Policy

Nginx/Caddy/Traefik should be the public ingress layer.

The reverse proxy should only route each domain to its intended backend.

The web root should point only to the public directory when static serving is required.

## 9.3 Outbound Network Policy

Outbound access should be minimized for high-risk workers.

Build jobs are a special case because package managers often need internet access. However, build jobs should not automatically receive access to internal metadata services, private service networks, database ports, or secrets services.

The agent should flag:

- Apps listening on public interfaces unnecessarily
- Internal admin dashboards exposed publicly
- Databases bound to `0.0.0.0`
- Redis/Memcached bound publicly
- Docker API exposed over TCP
- Package/build workers with unnecessary access to internal services

---

# 10. Secrets Policy

## 10.1 Secrets Placement

Secrets should be stored only in app-specific protected locations, such as:

```text
/srv/apps/<app>/shared/.env
/etc/<app>/env
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

## 10.2 Secrets Access

Only the specific service user or service requiring a secret should be able to read it.

The deploy worker should not pass all global secrets to every build job.

Build jobs should receive only the minimum secrets required for that exact job.

## 10.3 Findings

The agent should flag:

- `.env` files world-readable
- `.env` files readable by unrelated service users
- secrets under public web roots
- secrets copied into release artifacts
- secrets present in logs
- secrets exposed through systemd unit files readable by all users
- SSH private keys readable by service users
- package manager tokens readable by untrusted build scripts

---

# 11. Web Server Policy

## 11.1 Nginx/Caddy User Separation

The web server should run as its own user, such as:

```text
nginx
www-data
caddy
```

It should not run as a service user unless explicitly justified.

## 11.2 Static File Access

The web server should only read public/static directories needed for serving.

Good:

```text
root /srv/apps/app1/current/public;
```

Bad:

```text
root /srv/apps/app1/current;
root /srv/apps/app1;
root /srv/apps;
```

## 11.3 Uploads

Uploaded files should not be executable.

Upload directories should avoid script execution. For PHP apps, ensure upload directories cannot execute PHP files.

The agent should flag:

- Web roots that expose app root
- Directory listing enabled unintentionally
- Upload directories that allow script execution
- Sensitive files accessible over HTTP, such as `.env`, `.git`, backups, SQLite DBs, or logs

---

# 12. Database and Local Service Policy

## 12.1 SQLite

If SQLite is used, database files should be app-specific and protected:

```text
/srv/apps/<app>/shared/database.sqlite
```

Permissions:

```text
0600 <service-user>:<service-group>
```

or, if deploy/backup group access is needed:

```text
0640 <service-user>:<restricted-group>
```

Other service users should not be able to read SQLite files.

## 12.2 Network Databases

Postgres/MySQL/MariaDB should bind only to localhost or private interfaces unless external access is intentionally required.

Each app should have a distinct DB user with least privilege.

Bad:

```text
all apps use root DB user
all apps share same database credentials
DB listens publicly
```

Good:

```text
app1 has app1_db_user
app2 has app2_db_user
DB listens on localhost/private network
```

## 12.3 Redis/Memcached

Redis and Memcached should not be publicly exposed.

If shared, logical separation should be used where possible, but for stronger isolation each app should have a separate instance or at least separate credentials/namespaces where supported.

---

# 13. Deployment and Build Worker Policy

## 13.1 Separate Deployment Service

Deployment orchestration should run as the BonesDeploy deploy user, not as root unless absolutely necessary.

Example:

```text
bonesdeploy
```

Where root actions are needed, use narrow, audited helper commands rather than broad sudo.

## 13.2 Atomic Releases

Deployments should use release directories and atomic symlink flips:

```text
/srv/apps/<app>/releases/<release-id>
/srv/apps/<app>/current -> releases/<release-id>
```

Service users should not mutate old releases or deployment metadata.

## 13.3 Build Isolation

Builds should occur in a staging/build workspace, not directly inside the public path or active release tree.

Example:

```text
/srv/apps/<app>/tmp/build-<job-id>
```

Build scripts should be run with:

- Dedicated build user or deploy user
- No sudo
- Dropped capabilities
- cgroup limits
- AppArmor profile
- Landlock restrictions where possible
- Minimal secrets
- Controlled network access

## 13.4 Findings

The agent should flag:

- Build scripts running as root
- Build scripts running as deploy user with broad access to all apps
- Package install scripts inheriting production secrets
- Build workspace shared across apps
- Public path writable during build
- Current symlink writable by service user
- Deployment SSH keys readable by service user

---

# 14. Containers and Docker Policy

## 14.1 Docker Socket

Access to the Docker socket is equivalent to root-level control of the host in many practical deployments.

The agent should treat the following as critical unless explicitly justified:

```text
service user can access /var/run/docker.sock
container mounts /var/run/docker.sock
service user is in docker group
```

## 14.2 Container Hardening

Containers should use:

```text
--cap-drop=ALL
--security-opt no-new-privileges:true
read-only root filesystem where practical
specific writable volumes only
non-root user
memory limits
pids limits
CPU limits
AppArmor/seccomp profiles
```

## 14.3 Volume Policy

Containers should not mount broad host paths such as:

```text
/
/home
/root
/etc
/srv/apps
/var/run/docker.sock
```

unless there is a specific administrative container with strong justification.

---

# 15. SSH Policy

## 15.1 SSH Access

SSH should use key-based authentication.

Recommended:

```text
PasswordAuthentication no
PermitRootLogin no
PubkeyAuthentication yes
```

Root login should be disabled unless there is a documented emergency access model.

## 15.2 SSH Keys

Deployment SSH keys should be readable only by the deploy user or root.

Service users should not be able to read deployment keys.

The agent should flag:

- Private keys world-readable or group-readable by broad groups
- App users with SSH private keys
- Shared SSH keys across unrelated apps
- Root login enabled without justification
- Password auth enabled on internet-facing SSH without justification

---

# 16. Logging and Observability Policy

## 16.1 Logs Should Not Leak Secrets

Application logs should not contain:

- `.env` contents
- Authorization headers
- API tokens
- OAuth secrets
- Database passwords
- Private keys
- Session cookies
- Full request bodies containing credentials

## 16.2 Log Permissions

Logs should be writable by the app or captured by journald, but not broadly writable by unrelated users.

Other service users should not be able to read sensitive logs.

## 16.3 Audit Signals

The system should preserve logs useful for security review:

- systemd journal for app services
- auth logs
- sudo logs
- AppArmor denials
- web server access/error logs
- deployment logs

The agent should flag missing or disabled logs for critical services.

---

# 17. Backup Policy

## 17.1 Backup Access

Backup jobs often need broad read access. Therefore, backup users/services should be treated as high-privilege.

Backup credentials should not be readable by service users.

## 17.2 Backup Storage

Backups should not be stored inside public_path or app directories readable by service users.

The agent should flag:

- `.tar`, `.zip`, `.sql`, `.sqlite`, `.bak`, `.dump` files under public directories
- backups readable by unrelated service users
- backups containing secrets without encryption
- backup credentials available to runtime apps

---

# 18. Package Manager and Runtime Policy

## 18.1 Package Managers in Production

Production services should not generally need to run package managers at runtime.

The agent should flag if service users can run or write to package manager global locations unnecessarily:

```text
npm global dirs
composer global auth/cache
pip global locations
gem global paths
cargo global paths
```

## 18.2 Build vs Runtime Separation

Dependencies should be installed during build/deploy, not by the runtime web process.

Service users should not need write access to:

```text
node_modules
vendor
.venv
bundle
```

unless the app is explicitly designed for dynamic plugin/dependency installation.

---

# 19. Agent Audit Checklist

An auditing agent should collect and compare at least the following:

## 19.1 System Overview

```bash
uname -a
lsb_release -a || cat /etc/os-release
systemctl --version
mount
findmnt
```

## 19.2 Users and Groups

```bash
getent passwd
getent group
sudo -l -U <service-user>
groups <service-user>
```

Check whether service users:

- have login shells
- belong to sudo/wheel/docker groups
- own or can read unrelated app directories
- can read SSH keys or secrets

## 19.3 Filesystem Layout

```bash
ls -lah /srv
ls -lah /srv/apps
find /srv/apps -maxdepth 3 -type d -printf '%m %u %g %p\n'
find /srv/apps -name '.env' -o -name '*.sqlite' -o -name '*.db' -o -name '*.pem' -o -name '*.key'
```

Check permissions and ownership.

## 19.4 Web Server Exposure

```bash
ss -tulpen
nginx -T 2>/dev/null || true
caddy validate 2>/dev/null || true
```

Check exposed ports, roots, proxy targets, and accidental public access.

## 19.5 systemd Hardening

```bash
systemctl list-units --type=service
systemctl cat <service>
systemctl show <service>
systemd-analyze security <service>
```

Compare each app service to this policy.

## 19.6 AppArmor

```bash
aa-status
cat /sys/module/apparmor/parameters/enabled 2>/dev/null
ls -lah /etc/apparmor.d
```

Check whether app services are confined and enforcing.

## 19.7 Capabilities

```bash
getpcaps <pid>
capsh --print
systemctl show <service> -p CapabilityBoundingSet -p AmbientCapabilities
```

Check for unnecessary capabilities.

## 19.8 cgroups

```bash
systemctl show <service> -p MemoryMax -p MemoryHigh -p CPUQuotaPerSecUSec -p TasksMax
systemd-cgls
systemd-cgtop
```

Check resource isolation.

## 19.9 Docker/Containers

```bash
getent group docker
ls -l /var/run/docker.sock
docker ps --format '{{.Names}} {{.Image}} {{.Ports}}'
docker inspect <container>
```

Check socket exposure, capabilities, mounted volumes, users, and security options.

## 19.10 Secrets Search

Search for likely secret exposure carefully:

```bash
find /srv/apps -type f \( -name '.env' -o -name '*.pem' -o -name '*.key' -o -name '*credentials*' -o -name '*secret*' \) -printf '%m %u %g %p\n'
```

Do not print secret contents into logs or reports unless explicitly requested. Report paths and permissions only.

---

# 20. Severity Guide

## Critical Findings

- App service runs as root without necessity
- Service user has sudo/wheel/docker access
- App can read another app's secrets or database
- Docker socket exposed to app/container
- Public web access to `.env`, `.git`, database files, backups, or private keys
- SSH private keys readable by service users
- Database/Redis/Memcached publicly exposed without strong auth/firewalling
- Build scripts run as root with untrusted input

## High Findings

- All apps run as one shared Unix service user
- AppArmor disabled or app services unconfined
- No cgroup limits on untrusted workers
- Service user can write to release/source code directories
- Broad write access to `/srv/apps/**`
- `NoNewPrivileges=false` or absent for app services
- Dangerous capabilities granted unnecessarily
- Secrets passed broadly to build jobs

## Medium Findings

- No `ProtectSystem` or weak systemd hardening
- No `PrivateTmp`
- Logs readable by unrelated service users
- Upload directories allow script execution
- App backend binds publicly instead of localhost/private socket
- App has broader read access than necessary

## Low Findings

- Layout uses `/var/www` but is otherwise well-isolated
- Missing documentation for exceptions
- Excessively broad but non-sensitive read access
- Inconsistent ownership naming conventions

---

# 21. Acceptable Exceptions

Exceptions are allowed when they are explicit, narrow, and documented.

Each exception should include:

```yaml
exception:
  service: <service-name>
  control: <policy-control-being-relaxed>
  reason: <why-this-is-required>
  compensating_controls:
    - <control>
  review_date: <date>
```

Examples:

```yaml
exception:
  service: image-processor.service
  control: MemoryMax higher than default
  reason: Large image transformations need more memory
  compensating_controls:
    - Runs as dedicated user
    - AppArmor enforced
    - No sudo
    - PrivateTmp enabled
  review_date: 2026-08-01
```

---

# 22. Desired End State Summary

A well-configured BonesDeploy server should look like this:

```text
/srv/apps/<app> layout per app
one Unix service user per app
no service users with sudo/docker/root access
service users cannot write immutable release code
only app-specific writable dirs are writable
secrets readable only by intended service users
nginx/caddy exposes only public_path and reverse proxies to local backends
systemd hardening enabled per service
capabilities dropped by default
NoNewPrivileges enabled
cgroup limits set
AppArmor enabled and enforcing
Landlock used for risky child jobs where practical
seccomp used where practical
logs and backups protected
no public databases/caches/admin services
no Docker socket exposure to apps
```

The most important practical principle:

```text
A compromised app should only be able to damage itself and the small set of resources it explicitl
