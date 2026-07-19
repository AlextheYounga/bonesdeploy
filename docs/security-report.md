# BonesDeploy / BonesInfra Security Report

**Scope:** Compare the current BonesDeploy + BonesInfra implementation against the
Linux hardening resources in `/home/alex/Work/skills/security/` and the project's
own stated security posture in `docs/PROJECT.md`. Research only — no code changes.

**Sources compared:**
- `How-To-Secure-A-Linux-Server` (HTSL), `awesome-security-hardening`,
  `awesome-security`, `awesome-embedded-linux-security`,
  `awesome-pentest-cheat-sheets`, `awesome-agent-skills-security`
- BonesInfra setup plan: `bonesinfra/src/bonesinfra/deploys/setup/plan.py`
- BonesInfra assets under `bonesinfra/src/bonesinfra/assets/`
- BonesRemote Rust sources under `crates/bonesremote/`
- BonesDeploy Rust sources under `crates/bonesdeploy/`
- Provisional Lynis audit shipped at `bonesinfra/docs/lynis_report.txt`

The summary table below uses **[+]** for controls already implemented, **[~]**
for partial / inconsistent with the standard practice, **[-]** for missing, and
**[!]** for an outright defect or regression vs. the documented posture.

---

## Summary

| Area | Status | Notes |
|------|--------|-------|
| Sudoers narrowing | [~] | Drop-in is `0440`, `visudo -c` validated, command list is narrow — but `--site *` wildcard allows cross-site invocation on a shared host. |
| SSH host-key verification | [!] | Control-plane push uses `StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null` (MITM exposure); `openssh` paths use TOFU only. |
| SSH server hardening | [-] | No `sshd_config` template is provisioned by BonesInfra. `PermitRootLogin`, `PasswordAuthentication`, `AllowGroups`, KEX/MAC/cipher hardening, `MaxAuthTries`, `LoginGraceTime` are all left at Debian defaults. Lynis flagged this. |
| Kernel / sysctl hardening | [-] | No `/etc/sysctl.d/` drop-in is provisioned. Only the `algif_aead` modprobe disable is shipped (CVE-2026-31431). |
| Firewall (UFW) | [+] | `ufw` provisioned with default-deny incoming, explicit allow for HTTP/HTTPS, optional SSH rate-limit and CIDR allow-list. |
| Fail2ban | [~] | `jail.local` ships with `sshd` jail only; no `banaction=ufw`, no nginx/php-fpm jails, no `ignoreip`. |
| AppArmor | [+] | Per-project nginx profile + per-runtime app profile, enforced via `aa-enforce`. `deny /root/**`, `deny /etc/ssh/**`. |
| Systemd service hardening | [+] | Generated `.service` units are very tight: `NoNewPrivileges`, `ProtectSystem=strict`, `ProtectHome`, `PrivateTmp`, `PrivateDevices`, `ProtectKernel*`, `RestrictNamespaces`, `LockPersonality`, `RestrictRealtime`, `SystemCallArchitectures=native`, `CapabilityBoundingSet=` empty, `AmbientCapabilities=` empty, `ReadWritePaths` scoped. Lynis `systemd-analyze security` rates `makebabies-nginx.service` at 3.2 (PROTECTED) vs 9.6 for stock `nginx.service`. |
| Unattended-upgrades | [~] | Installed and enabled, but `Automatic-Reboot` is `false` (kernel security updates only take effect on next manual reboot), and no `Mail`/`MailOnlyOnError`. |
| Podman / container hardening | [~] | Rootless Podman, `--security-opt=no-new-privileges`, build cache `0700`, build container is `Drop`-cleaned. Missing: `--cap-drop=all` (explicitly *not* set), no `--read-only`, no `--network=none`/`--network` isolation, no image-signing/scan. |
| Nginx hardening | [-] | Per-site nginx config ships no `server_tokens off`, no `X-Frame-Options`, no `X-Content-Type-Options`, no `Referrer-Policy`, no `Content-Security-Policy`, no `Permissions-Policy`, no TLS cipher hardening, no rate-limit zones. |
| File permissions / ownership | [+] | Three-identity model enforced; `releases/` is `2750 root:<runtime_group>`; `shared/` is `0750` runtime-owned; `shared/.env` is `0640`. Symlink escape in build tree is validated and rejected. Build cache `0700`. |
| Setgid on release subtree | [~] | `docs/PROJECT.md:333` claims "All release artifacts are created with the setgid bit". `releases/` parent is `2750`, but `bonesremote` chmods each child dir `0750` and file `0640` *without* `0o2000`, so setgid does not propagate. Functional ownership is correct (explicit `chown` to `root:<runtime_group>`), but the documented claim diverges. |
| Auditd / AIDE / file integrity | [-] | Not provisioned. Lynis flagged `auditd` as NOT FOUND. |
| Bootloader / firmware | [-] | No GRUB password, no UEFI/Secure Boot verification. Lynis flagged GRUB and UEFI boot disabled. |
| Password / login policy | [-] | No `libpam-pwquality`, no `login.defs` `UMASK`, no per-deploy-user umask. Lynis flagged `umask`, password aging, failed-login logging as disabled/suggestion. |
| Users / accounts | [+] | Strong three-identity model: deploy user `git` (home, no sudo beyond narrow list), runtime user per-project (system, `nologin`, `home=/nonexistent`), per-project build user (`nologin`, lingering on for systemd user manager). `authorized_keys` is `0600`, `.ssh` is `0700`. |
| Logging / journald | [~] | Services write to `journal`; `fail2ban` uses `systemd` backend. No `Storage=persistent` in `journald.conf`, no `logwatch`, no remote logging (Lynis flagged remote logging NOT ENABLED). |
| Build-user resource limits | [+] | `user-<uid>.slice` is constrained via root-owned drop-in with `CPUQuota`, `MemoryHigh`, `MemoryMax`. Configurable in `bones.toml`. |
| Deployment lock | [+] | `flock`-based lock at `<sites_root>/.<site>.deployment.lock` outside the replaceable dataset; shared by `deploy`, `site import`, `release kill`. |
| Secrets (GPG at rest) | [~] | Local project GPG key uses `%no-protection` (no passphrase). Acceptable for single-user workstation threat model, weaker for shared workstations. |
| Secrets (transit + on-server) | [~] | Pushed over SSH to root, stored `0640 root:<runtime_group>`. Plaintext temp file via `mktemp` has no `trap rm -f` on failure path. |
| Service restart verification | [!] | `bonesremote service restart` only checks `systemctl restart` exit code. No `is-active` poll, no journal tail, no per-service check. PROJECT.md implies verified restart; implementation doesn't. |
| Sudoers coverage vs. CLI | [~] | `release kill` is **not** in the sudoers drop-in, so it is only reachable via direct root SSH. Inconsistent with `release drop-failed` / `release prune` being sudoable. |
| `release finalize` setgid | [~] | See "Setgid on release subtree" above. |
| `site import` repo_path safety | [!] | `site.rs:59-72` writes the baked `post-receive` hook to `<repo_path>/hooks/post-receive` from the imported `bones.toml`. `repo_path` is validated against the site *name* only, not against a known parent. Untrusted datasets can target arbitrary writable directories. |
| `hook post-receive` ref validation | [~] | Trusts `newrev` if the branch name matches `cfg.branch`. No `git rev-parse --verify` before delegating to `deploy::run_full`. Defense-in-depth gap (downstream `git archive` would fail) rather than direct exploit. |
| `doctor` sudo negative check | [~] | Verifies the wildcard grant matches, but does not assert that `git` *cannot* run `bonesremote deploy *`, `release activate *`, etc. A misconfigured broader grant would still pass. |

---

## Findings by area

### 1. Sudoers
**Implemented:** `/etc/sudoers.d/bonesdeploy` rendered from
`bonesinfra/src/bonesinfra/assets/sudoers/bonesdeploy.j2`, mode `0440`, `root:root`,
validated with `visudo -c -f` (auto-removed on failure). Grants the deploy user
(`git`) passwordless access to exactly five `bonesremote` subcommands:

```
git  ALL=(root) NOPASSWD: /usr/local/bin/bonesremote hook post-receive --site *, \
                            /usr/local/bin/bonesremote service restart    --site *, \
                            /usr/local/bin/bonesremote release rollback   --site *, \
                            /usr/local/bin/bonesremote release drop-failed --site *, \
                            /usr/local/bin/bonesremote release prune       --site *
```

Notably **not** granted: `deploy`, `release stage`, `release checkout`,
`release build`, `release promote`, `release finalize`, `release wire`,
`release prepare`, `release activate`, `release kill`, `site import`. Those
are reachable only by direct root SSH or in-process via `hook post-receive`
which runs `deploy::run_full`.

**Gaps vs. standard practice:**
- **`--site *` wildcard allows cross-site invocation.** On a shared host
  with multiple projects, a compromised `git` user (or anyone able to push
  to any site's bare repo) can invoke `service restart --site <other-site>`
  or `release rollback --site <other-site>` for a project they don't own.
  The earlier BonesInfra migration plan
  (`bonesinfra/docs/plans/migration/01_security_architecture_problems.md:212`)
  recommended tying the rule to a registry-path glob, but the rendered rule
  still uses `--site *`.
- The in-process `cfg.project_name == site` check inside `service::run`
  only validates the named site's *state*, not that the *caller* is
  authorized for that site.
- HTSL/awesome-security-hardening recommend restricting sudo to a named
  group (`%sudousers`), pinning command paths, and avoiding `NOPASSWD:`
  except for narrow lists. The bonesinfra rule does pin paths and is narrow,
  but it is per-user (`git`) rather than per-group, and the wildcard on
  `--site` weakens the narrowing.

**Recommendation:** Either (a) emit one `Cmds_Alias` per registered site and
a per-site line `git ALL=(root) NOPASSWD: <alias>` (so `--site <name>` is
literal, not `*`), or (b) gate by a per-site `runas` group.

### 2. SSH
**Server side (sshd_config):** BonesInfra **does not provision any
`sshd_config` template.** Every HTSL/awesome-security-hardening recommendation
in this area is unaddressed:
- `PermitRootLogin no` — *not set.* The deploy flow *requires* root SSH
  (`bonesdeploy deploy` and `secrets push` both SSH as `root`), so
  `PermitRootLogin prohibit-password` (the Debian default) is the only
  gate. Switching to `PermitRootLogin no` would break the documented flow
  unless `bonesdeploy`/`secrets` move to the `git` user + sudo.
- `PasswordAuthentication no` — left at Debian default (yes).
- `AllowGroups sshusers` — not set.
- Kex / Ciphers / MACs / `RequiredRSASize` — left at defaults.
- `MaxAuthTries`, `LoginGraceTime`, `MaxStartups`, `LogLevel VERBOSE` —
  not set. Lynis flagged all of these as `SUGGESTION`.

**Client side (workstation → host):**
- `crates/bonesdeploy/src/infra/ssh.rs:10-23` uses the `openssh` crate with
  `KnownHosts::Accept` (TOFU: adds unknown keys, does not reject on
  mismatch by default). No password auth, no key pinning.
- `crates/bonesdeploy/src/infra/ssh.rs:35-41` (`external_command`) builds
  a raw `ssh` `Command` with **`-o StrictHostKeyChecking=no
  -o UserKnownHostsFile=/dev/null`**, used by `commands/push_state.rs:30`
  to stream the `.bones/` archive into `bonesremote site import`. This
  accepts any host key on every push — a MITM on the control-plane push
  can substitute deployment scripts that later run as the runtime user
  and inside the build container.
- `secrets push` and `deploy` SSH as `cfg.ssh_user` which defaults to
  `root` (`docs/PROJECT.md:90`). The workstation's root SSH key is the
  deployment credential; a compromised workstation is root on every
  configured deploy host. The sudoers narrowing on `git` does not help
  here because root is the privileged path.

**Recommendation:** (a) Replace `external_command` with `openssh`-crate
sessions using `KnownHosts::Add` (TOFU) at minimum, or refuse-on-mismatch
after first connect; (b) ship a hardened `sshd_config` template via
BonesInfra (or document why it is intentionally left to the operator);
(c) consider moving `deploy` and `secrets push` to the `git` user +
narrowed sudo so `PermitRootLogin no` becomes viable.

### 3. Kernel / sysctl
**Implemented:** Only the `algif_aead` modprobe disable
(`bonesinfra/src/bonesinfra/deploys/setup/kernel_hardening.py`,
`assets/modprobe/disable-algif.conf.j2`) for CVE-2026-31431.

**Missing** (HTSL `linux-kernel-sysctl-hardening.md`, awesome-security-hardening):
`kernel.kptr_restrict=2`, `kernel.randomize_va_space=2`, `kernel.sysrq=0`,
`kernel.ctrl-alt-del=0`, `fs.suid_dumpable=0`, `fs.protected_hardlinks=1`,
`fs.protected_symlinks=1`, `vm.mmap_min_addr=4096`, all
`net.ipv4.conf.*.{accept_redirects,secure_redirects,send_redirects,
accept_source_route,rp_filter,log_martians}`, `net.ipv4.ip_forward=0`,
`net.ipv4.tcp_syncookies=1`, `tcp_rfc1337=1`,
`net.ipv6.conf.*.{accept_ra,accept_redirects,accept_source_route,
forwarding}`, `/proc hidepid=2`.

Lynis flagged `umask (/etc/login.defs)`, `/proc hidepid`, `/home` and
`/var` mount options, USB-storage and firewire ohci modules not disabled.

**Recommendation:** Add a `bonesinfra/deploys/setup/sysctl.py` that drops a
prioritized `/etc/sysctl.d/99-bonesdeploy.conf` with the standard hardening
set. The `algif_aead` modprobe disable is fine as-is.

### 4. Firewall (UFW)
**Implemented:** `bonesinfra/src/bonesinfra/deploys/setup/firewall.py`
provisions UFW with `default deny incoming`, `default allow outgoing`,
explicit allow for HTTP/HTTPS, and optional `limit` (rate-limit) or
`allow` for SSH, optional CIDR allow-list for SSH, optional SSH rate-limit.
Configurable via `[runtime]` keys (`firewall_enabled`, `firewall_ssh_rate_limit`,
`firewall_ssh_allowed_cidrs`, etc.). Defaults: `firewall_enabled=True`,
rate-limit off, no CIDR filter.

**Gaps:**
- Default outgoing is `allow` — HTSL recommends `default deny outgoing`
  with allow-by-exception (DNS, NTP, HTTP/HTTPS, SMTP). A compromised
  runtime service could exfiltrate over arbitrary ports.
- `ufw limit` (rate-limit SSH) is opt-in; the standard recommendation
  is to make it the default when SSH is exposed.

**Recommendation:** Make `firewall_ssh_rate_limit` default `True` for
non-CIDR-locked SSH. Consider offering a `firewall_default_outgoing=deny`
opt-in for stricter deployments.

### 5. Fail2ban
**Implemented:** `bonesinfra/src/bonesinfra/deploys/setup/fail2ban.py`
installs `/etc/fail2ban/jail.local` from `assets/fail2ban/jail.local.j2`:
```
[DEFAULT]
bantime = 1h
findtime = 10m
maxretry = 5
backend = systemd
[sshd]
enabled = true
port = {{ ssh_port }}
logpath = %(sshd_log)s
```

**Gaps vs. HTSL/awesome-security-hardening:**
- `banaction` is not set — defaults to `iptables-multiport` on Debian, not
  `ufw`, so bans don't show up in `ufw status`.
- No `ignoreip` (loopback + trusted LAN).
- No `destemail` / `action` — no alerting.
- Only `sshd` jail. No `nginx-limit-req`, `nginx-botsearch`, `php-fpm`,
  or per-app jails even though nginx/php-fpm auth logs are predictable.
- Consider CrowdSec (community blocklist, IPv6, modern replacement).

**Recommendation:** Add `banaction = ufw` and an `ignoreip` loopback +
operator CIDR. Add a small library of per-runtime jail snippets that
`runtime apply` can install alongside the runtime service.

### 6. AppArmor / MAC
**Implemented:**
- `bonesinfra/src/bonesinfra/deploys/runtime/apparmor.py` renders
  `/etc/apparmor.d/bonesdeploy-<project>-nginx` from
  `assets/apparmor/project-nginx-profile.j2`, loads with
  `apparmor_parser -r`, enforces with `aa-enforce`.
- `bonesinfra/src/bonesinfra/runtimes/common/apparmor.py` renders a
  per-runtime app profile from `runtimes/common/assets/app-profile.j2`,
  loads with `apparmor_parser -r -T -W`, enforces with `aa-enforce`.
- `bonesremote doctor` verifies AppArmor is enabled and active.
- Profiles deny `/root/**` and `/etc/ssh/**` reads.
- `kernel.yama.ptrace_scope` is not set by BonesInfra (HTSL recommends
  `1` or `2`).

**Gaps:** No seccomp filter applied to services beyond
`SystemCallArchitectures=native` in the systemd unit (HTSL recommends a
`SystemCallFilter=@system-service` allow-list).

**Recommendation:** Add `SystemCallFilter=@system-service` and
`SystemCallErrorNumber=EPERM` to the generated service units, plus
`RestrictSUIDSGID=yes` and `MemoryDenyWriteExecute=yes`. Consider
`kernel.yama.ptrace_scope=2` in the sysctl drop-in.

### 7. Systemd service hardening
**Implemented** (in both `runtimes/common/assets/app.service.j2` and
`assets/nginx/site-nginx.service.j2`):

| Directive | Present? |
|-----------|----------|
| `User=` / `Group=` (per-project) | ✓ |
| `NoNewPrivileges=yes` | ✓ |
| `ProtectSystem=strict` | ✓ |
| `ProtectHome=yes` | ✓ |
| `PrivateTmp=yes` | ✓ |
| `PrivateDevices=yes` | ✓ |
| `ProtectKernelTunables=yes` | ✓ |
| `ProtectKernelModules=yes` | ✓ |
| `ProtectControlGroups=yes` | ✓ |
| `RestrictNamespaces=yes` | ✓ |
| `LockPersonality=yes` | ✓ |
| `RestrictRealtime=yes` | ✓ |
| `SystemCallArchitectures=native` | ✓ |
| `CapabilityBoundingSet=` (empty) | ✓ |
| `AmbientCapabilities=` (empty) | ✓ |
| `RestrictAddressFamilies=` (runtime-scoped) | ✓ |
| `ReadOnlyPaths=` / `ReadWritePaths=` | ✓ |
| `AppArmorProfile=` | ✓ |
| `Restart=always` | ✓ |

**Missing** vs. HTSL/awesome-security-hardening:
- `ProtectClock=yes`
- `ProtectHostname=yes`
- `ProtectKernelLogs=yes`
- `RestrictSUIDSGID=yes`
- `MemoryDenyWriteExecute=yes`
- `RemoveIPC=yes`
- `SystemCallFilter=@system-service` with `SystemCallErrorNumber=EPERM`
- `LimitNOFILE=` / `LimitNPROC=` / `TasksMax=`
- `IPAddressAllow=` / `IPAddressDeny=` (only set for nginx when
  `nginx_ip_loopback_only` is true)
- `PrivateUsers=yes` (probably not viable for nginx binding to a unix
  socket, but worth evaluating per runtime)

Lynis rates the generated `makebabies-nginx.service` at **3.2 (PROTECTED)**,
vs. stock `nginx.service` at **9.6 (UNSAFE)** — a strong win, but not yet
at the ~0.5 floor achievable with the directives above.

**Recommendation:** Add the missing directives above to both templates;
re-evaluate with `systemd-analyze security`.

### 8. Unattended-upgrades
**Implemented:** `bonesinfra/src/bonesinfra/deploys/setup/unattended_upgrades.py`
installs `/etc/apt/apt.conf.d/20auto-upgrades` (`Update-Package-Lists=1`,
`Unattended-Upgrade=1`) and `50unattended-upgrades.j2` with Allowed-Origins
covering `distro_id:codename`, `-security`, ESMApps, ESM-infra; removes
unused kernel packages and unused dependencies; `DevRelease=false`.

**Gaps:**
- `Automatic-Reboot=false` and `Automatic-Reboot-WithUsers=false`. Kernel
  security updates therefore only take effect on the next manual reboot.
  HTSL recommends `Automatic-Reboot=true` + `Automatic-Reboot-WithUsers=true`
  when `/var/run/reboot-required` is set.
- No `Mail` / `MailOnlyOnError` — no operator alerting.
- `AutoFixInterruptedDpkg` not set.

**Recommendation:** Expose `Mail`/`MailOnlyOnError` via `[runtime]` keys
(default off, but document); decide whether `Automatic-Reboot` should
default on. At minimum, surface `reboot-required` state in
`bonesremote doctor`.

### 9. Podman / container hardening
**Implemented** (in `crates/bonesremote/src/release/script_runner/build/container.rs`):
- Rootless Podman under the per-project `<site>-build` user.
- Build user `nologin`, lingering on, dedicated home `0700`, dedicated
  cache `0700`, dedicated subuid/subgid ranges, dedicated `storage.conf`.
- Container launched via
  `systemd-run --machine=<site>-build@ --quiet --user --collect --unit <name> --service-type=notify --property=NotifyAccess=all --property=KillMode=none podman run -d --pull=never --sdnotify=conmon --cgroups=no-conmon --security-opt=no-new-privileges --workdir=/workspace/source --name <name>`.
- Only two volumes: `<build_context>:/workspace/source` and
  `<build_cache_dir>:/workspace/cache:rw`. `.env`, `shared/`, `current`,
  `releases/`, the bare repo, and bonesremote control-plane paths are
  **not** mounted.
- Deployment bundle is streamed into the container's disposable
  filesystem via `tar | podman exec -i ... tar -x`, not bind-mounted.
- `Drop` impl removes the container even on panic.
- Build scripts run as `bash -c "umask 0002; exec bash -s"` with the
  script piped on stdin.
- `[build].vars` subset of `shared/.env` injected via `--env`; the full
  `.env` is not visible to the build.

**Gaps vs. awesome-security-hardening / CIS Docker Benchmark:**
- **`--cap-drop=all` is explicitly not set** (`container.rs:284` asserts
  its absence). The container runs with Podman's rootless default
  capability set (CHOWN, DAC_OVERRIDE, FOWNER, SETGID, SETUID,
  NET_BIND_SERVICE, KILL, etc.). Rootless narrows the surface, but
  `--cap-drop=all` with explicit `--cap-add` for only what
  `buildpack-deps:bookworm` needs would be tighter.
- No `--read-only` rootfs. Build scripts can write anywhere in the
  container FS. (The cache and source mounts are writable; everything
  else could be read-only.)
- No `--network=none` or isolated `--network` for build steps that
  don't need network egress. `buildpack-deps:bookworm` builds usually
  need network for `apt`/`pip`/`npm`, but a per-script opt-out would
  reduce exfiltration risk.
- No `--pids-limit`, `--memory`, `--cpus` on the container itself (the
  build *slice* limits the user, but a runaway process can still
  fork-bomb within the container).
- No image signing or scan (Trivy/Grype) before deploy.
- No `--security-opt no-new-privileges` is *set* — good.

**Recommendation:** Set `--cap-drop=all` and enumerate the small set of
capabilities `buildpack-deps:bookworm` actually needs. Add `--pids-limit`
and consider `--read-only` with explicit `--tmpfs` for build temp dirs.
Document an opt-in for `--network=none` on builds that don't fetch deps.

### 10. Nginx hardening
**Implemented:** Per-site nginx config (`app-site-nginx.conf.j2`,
`router.conf.j2`) is functional but ships **none** of the standard
response headers:

| Header | Present? |
|--------|----------|
| `server_tokens off` | ✗ |
| `X-Frame-Options SAMEORIGIN` | ✗ |
| `X-Content-Type-Options nosniff` | ✗ |
| `X-XSS-Protection` | ✗ |
| `Referrer-Policy strict-origin` | ✗ |
| `Content-Security-Policy` | ✗ |
| `Permissions-Policy` | ✗ |
| TLS cipher hardening (TLS 1.2/1.3 only, ECDHE+AEAD) | ✗ (only `listen 443 ssl`) |
| `ssl_protocols`, `ssl_ciphers`, `ssl_prefer_server_ciphers` | ✗ |
| HSTS (`Strict-Transport-Security`) | ✗ |
| Rate-limit zones (`limit_req_zone` / `limit_conn`) | ✗ |
| `add_header` always | ✗ |

Lynis confirmed nginx is the only thing in the stack with no insecure
protocols and no debug logging, but the response-header posture is empty.

**Recommendation:** Add a `security_headers` snippet included by both
`router.conf.j2` and `app-site-nginx.conf.j2` with the standard headers
above; add TLS hardening to `router.conf.j2` when `nginx_ssl_enabled`;
add HSTS when SSL is on; add an optional `limit_req_zone` for the app
proxy.

### 11. File permissions / ownership
**Implemented (strong):**
- `releases/` is `2750 root:<runtime_group>` (`directories.py:63-69`).
- `shared/` is `0750 <runtime_user>:<runtime_group>` (`directories.py:71-77`).
- `shared/.env` is `0640 <runtime_user>:<runtime_group>` on provisioning
  and `0640 root:<runtime_group>` after `secrets push` (`secrets.rs:141-145`).
- Placeholder release is `0750 root:<runtime_group>`.
- Build user home `0700`, build cache `0700`, `.config/containers` `0700`.
- `.ssh/authorized_keys` is `0600`, `.ssh` is `0700` (`users.py:255-271`).
- Symlink escape in build tree is validated and rejected
  (`tree.rs:70-82`).
- `current` is a symlink to a release owned by `root:<runtime_group>`.

**Gaps:**
- **Setgid not propagated into the release subtree.** PROJECT.md claims
  setgid on `releases/` is inherited by all artifacts; in practice
  `bonesremote` chmods every dir `0750` and file `0640` without `0o2000`
  (`tree.rs:117-123`). The runtime group is granted by explicit `chown`
  instead, so the *functional* outcome matches, but the doc claim is
  wrong and a future chmod regression would silently break group access.
- Default `umask` is not set globally. Build scripts run with
  `umask 0002` (injected by the container entry), but the deploy user's
  interactive shell umask is whatever Debian defaults to (Lynis:
  `umask (/etc/login.defs) [ SUGGESTION ]`).

**Recommendation:** Either propagate setgid (chmod dirs `0o2750`, files
`0o640`) to match the docs, or update PROJECT.md to say "runtime group
granted via explicit `chown`, setgid only on the `releases/` parent."
Add a `umask 0027` to `/etc/profile.d/` via BonesInfra for non-root shells.

### 12. Auditd / AIDE / file integrity
**Not implemented.** Lynis flagged `auditd NOT FOUND`, no accounting,
no sysstat. AIDE is not installed.

**Recommendation:** Install `auditd` with a minimal rule set watching
`/etc/passwd`, `/etc/shadow`, `/etc/sudoers*`, `/etc/ssh/sshd_config`,
`/root/.config/bonesremote/`, and `execve` on the build user. Install
`aide` with daily diff cron and add `/srv/sites/<project>/releases/`,
`/root/.config/bonesremote/`, `/etc/sudoers.d/` to monitored paths.

### 13. Bootloader / firmware
**Not implemented.** Lynis: `UEFI boot DISABLED`, GRUB2 password
`NONE`, `usb-storage` and `firewire ohci` not disabled, no Secure Boot
verification.

**Recommendation:** Document operator responsibilities (BonesInfra
can't safely install a GRUB password or change BIOS settings without
locking the operator out). At minimum, install a `modprobe.d` drop-in
disabling `usb-storage`, `firewire-ohci`, `cdfs`, and other unused
filesystems. Offer an opt-in GRUB-password templating step.

### 14. Password / login policy
**Not implemented.** No `libpam-pwquality`, no `login.defs UMASK`,
no `pam_tally2`/`pam_faillock` failed-login lockout. Lynis flagged
`PAM password strength tools [ SUGGESTION ]`, `umask (/etc/login.defs)
[ SUGGESTION ]`, `Logging failed login attempts [ DISABLED ]`,
`User password aging [ DISABLED ]`.

**Recommendation:** Install `libpam-pwquality` with a sane
`/etc/pam.d/common-password` line; set `UMASK 027` in `login.defs`;
enable `pam_faillock` for failed-login lockout. Most of this only
matters if password SSH is ever enabled, but the baseline is cheap.

### 15. Users / accounts
**Implemented (strong):**
- Deploy user `git` with `/bin/bash`, home, no password login by default
  (SSH key only).
- Per-project runtime user: `system`, `home=/nonexistent`,
  `shell=/usr/sbin/nologin`, no sudo, member of `<runtime_group>`.
- Per-project build user: `nologin`, lingering on, dedicated home, own
  subuid/subgid ranges, own group.
- Authorized-key copy: `0700` `.ssh`, `0600` `authorized_keys`,
  chown'd to the deploy user.

**Gaps:**
- No `sshusers`-style group gate on SSH (depends on `sshd_config`
  hardening above).
- No `sulogin` recovery path documented for the locked root model.

### 16. Logging / journald
**Implemented:** Services write `StandardOutput=journal`,
`StandardError=journal`. Fail2ban uses `backend = systemd`.

**Gaps:**
- `journald.conf` `Storage=persistent` is not enforced.
- No `logwatch` daily mail.
- No remote logging (Lynis flagged `remote logging NOT ENABLED`).
- No journald vacuum bound (`SystemMaxUse=`).

**Recommendation:** Provision a small `journald.conf` drop-in with
`Storage=persistent` and `SystemMaxUse=500M`. Document remote-logging
as operator responsibility.

### 17. Other / defense-in-depth
- **CVE-2026-31431 mitigation** (algif_aead modprobe disable): good,
  specific, and minimal.
- **Lynis** is run once and the report is checked into the repo, which
  is great. The report flags `PAM password strength`, `auditd`, GRUB
  password, `/proc hidepid`, USB-storage, remote logging — all
  addressable by the recommendations above.
- **`systemd-analyze security`** in the Lynis report confirms the
  generated per-site nginx unit is dramatically harder than stock
  (3.2 vs 9.6).
- **`BONES_BOOTSTRAP_SSH_USER`** env override
  (`crates/bonesdeploy/src/infra/bootstrap_ssh.rs:5-11`) lets the
  operator bootstrap as a non-root user. Good escape hatch; not the
  default.
- **No `cron`/`logwatch`/`apticron`** alerting — operators get no mail
  on security updates, failed bans, or AIDE diffs.

---

## Specific implementation defects (file:line)

These are concrete bugs/regressions, not "missing hardening". They
should be addressed before the next release regardless of the broader
hardening agenda.

1. **`crates/bonesdeploy/src/infra/ssh.rs:35-41` — `external_command`
   disables host-key verification.** Used by
   `crates/bonesdeploy/src/commands/push_state.rs:30` to stream the
   `.bones/` archive (which contains deployment scripts that will run
   as the runtime user and inside the build container) into
   `bonesremote site import`. MITM on this push substitutes the entire
   deployment bundle. Fix: route through the `openssh`-crate session
   builder, or set `StrictHostKeyChecking=accept-new` at minimum.

2. **`crates/bonesremote/src/commands/service.rs:22-32` — `service
   restart` only checks `systemctl restart`'s exit code.** No
   `systemctl is-active --wait` poll, no `is-failed` check, no journal
   tail. PROJECT.md:323-326 implies verified restart; the implementation
   does not verify. A unit that starts then exits 0 a few hundred
   milliseconds later is reported as "Restarted". The earlier
   `bonesinfra/docs/PROJECT.md:576-579` explicitly calls out this
   requirement: "BonesRemote should also verify every required service
   remains active after restarting."

3. **`crates/bonesremote/src/release/lifecycle/build/tree.rs:117-123`
   — `set_release_tree_identity` chmods dirs `0o750` and files `0o640`
   without `0o2000`, so the setgid bit on `releases/` is not inherited
   by `releases/<release>/` or its subdirectories.**
   `docs/PROJECT.md:333` claims otherwise. Either add `0o2000` to the
   directory chmods or update the docs.

4. **`crates/bonesdeploy/src/commands/secrets.rs:141-145` — `secrets
   push` writes the decrypted `.env` to a `mktemp` file on the remote
   and `mv`s it on success, with no `trap rm -f "$tmp"` on failure.**
   If `chown`/`chmod`/`mv` fails partway, the plaintext temp file
   remains under `<shared>/` on the server. Fix: wrap in
   `trap 'rm -f "$tmp"' EXIT` in the remote shell snippet.

5. **`crates/bonesremote/src/commands/site.rs:59-72` — `site import`
   writes the baked `post-receive` hook to `<repo_path>/hooks/post-receive`
   where `repo_path` comes from the freshly-imported `bones.toml`.**
   `repo_path` is validated against the site *name* only, not against a
   known parent. Untrusted datasets can target arbitrary writable
   directories with a `0755` file named `post-receive`. Fix: constrain
   `repo_path` to `/home/git/<project>.git` (or the configured
   `repo_parent`) before writing.

6. **`crates/bonesdeploy/src/commands/secrets.rs:203-239` — the project
   GPG key is generated with `%no-protection` (no passphrase).** Any
   process that can read `~/.config/bonesdeploy/_lib/gnupg/` (mode
   `0700`) can decrypt all project secrets. Acceptable for a
   single-user workstation threat model; weaker for shared
   workstations. Document the threat model, or offer an opt-in
   passphrase-protected key.

7. **`crates/bonesremote/src/commands/hook.rs:26-46` — `hook
   post-receive` trusts `newrev` from stdin if the branch name matches
   `cfg.branch`, without `git rev-parse --verify`.** Defense-in-depth
   gap: downstream `git archive` would fail for a bogus revision, but a
   hand-crafted `post-receive` invocation (or a forged ref update) can
   drive a deploy attempt for a non-existent revision.

8. **`crates/bonesremote/src/commands/release/kill.rs:19-74` —
   `release kill` is not in the sudoers drop-in.** Reachable only by
   direct root SSH, so `bonesdeploy releases kill <release>` from a
   `git`-triggered workflow fails at `ensure_root`. Inconsistent with
   `release drop-failed` and `release prune` being sudoable. Either
   add it to the sudoers template or document the omission.

9. **`crates/bonesremote/src/commands/doctor/system.rs:30-37` — the
   sudo check verifies the wildcard grant matches, but does not assert
   the negative (that `git` cannot run `bonesremote deploy *`, `release
   activate *`, etc.).** A misconfigured broader grant would still
   pass `doctor`. Fix: add negative assertions for the privileged
   subcommands that should *not* be sudoable.

10. **`bonesinfra/src/bonesinfra/assets/sudoers/bonesdeploy.j2:2` —
    `--site *` wildcard allows cross-site invocation on shared hosts.**
    A compromised `git` user can run `service restart --site
    <other-project>` for projects they don't own. Fix: emit a per-site
    `Cmds_Alias` with a literal `--site <name>` and one sudoers line
    per registered site.

---

## Prioritized recommendations

### High (fix before next release)
1. Tighten `external_command` SSH host-key verification (defect #1).
2. Add `systemctl is-active --wait` verification to `service restart`
   (defect #2).
3. Add `trap rm -f "$tmp"` to `secrets push` remote shell (defect #4).
4. Constrain `repo_path` in `site import` to the configured parent
   (defect #5).
5. Either fix setgid propagation or update PROJECT.md (defect #3).

### Medium (next hardening pass)
6. Provision a hardened `sshd_config` template (or document why it's
   intentionally operator-owned).
7. Add a `sysctl.d/99-bonesdeploy.conf` with the standard hardening
   set.
8. Add `banaction = ufw` + `ignoreip` + nginx/php-fpm jails to the
   fail2ban template.
9. Add the missing systemd directives (`ProtectClock`, `ProtectHostname`,
   `ProtectKernelLogs`, `RestrictSUIDSGID`, `MemoryDenyWriteExecute`,
   `RemoveIPC`, `SystemCallFilter=@system-service`, `LimitNOFILE`,
   `TasksMax`).
10. Add `--cap-drop=all` (with explicit `--cap-add`) and `--pids-limit`
    to the build container.
11. Add nginx security headers + TLS cipher hardening to the router
    and per-site templates.
12. Per-site sudoers alias instead of `--site *` (defect #10).

### Low (defense-in-depth)
13. Install `auditd` + `aide` and surface AIDE diffs in `doctor`.
14. Set `Automatic-Reboot=true` (opt-in) and `Mail`/`MailOnlyOnError`
    for unattended-upgrades.
15. Install `libpam-pwquality` + `pam_faillock` + `UMASK 027`.
16. Disable `usb-storage`, `firewire-ohci`, `cdfs` via `modprobe.d`.
17. Add `journald.conf` `Storage=persistent`, `SystemMaxUse=500M`.
18. Add `release kill` to the sudoers drop-in (defect #8) or document
    the omission.
19. Add negative sudo assertions to `doctor` (defect #9).
20. Add `git rev-parse --verify` to `hook post-receive` (defect #7).
21. Document the GPG `%no-protection` threat model (defect #6).

---

## Notes on methodology

- This report is based on static reading of the BonesDeploy/BonesInfra
  source tree and the six security resource repos under
  `/home/alex/Work/skills/security/`. No live hosts were scanned.
- The Lynis report at `bonesinfra/docs/lynis_report.txt` is from a
  `testbox` Debian 13 host and is used here as a sanity check, not as
  the primary source.
- "Standard practice" is the consensus of the six referenced
  awesome-lists + HTSL; where they disagreed, the more conservative
  option was used.
- Several PROJECT.md claims diverge from the implementation
  (setgid propagation, verified service restart). These are flagged
  as defects, not as documentation bugs, because the implementation
  is the source of truth and the docs claim the safer behavior.
