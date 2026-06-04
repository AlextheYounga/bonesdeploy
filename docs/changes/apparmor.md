# AppArmor Implementation Changes

## Overview

This branch implements per-project AppArmor confinement for per-site nginx, replacing broad runtime network access with explicit unix-socket-only permissions. The changes maintain Landlock as a supplemental job-time sandbox while making AppArmor the primary confinement mechanism for the nginx service boundary.

## What Changed

### 1. Ansible AppArmor Role

**Location:** `kit/remote/roles/apparmor/`

New Ansible role that provisions AppArmor for each project:

- Installs `apparmor` and `apparmor-utils` packages
- Enables and starts the `apparmor` service
- Verifies kernel AppArmor is enabled via `/sys/module/apparmor/parameters/enabled`
- Deploys per-project profile from template
- Loads profile with `apparmor_parser -r`
- Enforces profile with `aa-enforce`
- Verifies profile is loaded via `/sys/kernel/security/apparmor/profiles`
- Verifies the project profile appears as `(enforce)` in `/sys/kernel/security/apparmor/profiles`

**Profile naming:**
```
bonesdeploy-{{ project_name }}-nginx
```

**Profile path:**
```
/etc/apparmor.d/bonesdeploy-{{ project_name }}-nginx
```

### 2. Systemd Service Binding

**Location:** `kit/remote/nginx/site-nginx.service.j2`

Per-site nginx systemd unit now binds to AppArmor:

```ini
[Unit]
After=network.target apparmor.service
Requires=apparmor.service

[Service]
AppArmorProfile={{ apparmor_profile_name | default("bonesdeploy-" ~ project_name ~ "-nginx") }}
```

This ensures:
- AppArmor service must be active before nginx starts
- Profile is loaded before service execution
- Service runs confined under the per-project profile

### 3. AppArmor Profile Template

**Location:** `kit/remote/apparmor/project-nginx-profile.j2`

Profile characteristics:

**Network:**
- Allows `network unix stream` for socket communication
- No `network inet stream` or `network inet6 stream` by default
- Per-site nginx listens on unix socket; router nginx handles external traffic

**File access:**
- Read access to bonesremote and nginx binaries
- Read access to system paths: `/usr/**`, `/bin/**`, `/sbin/**`, `/lib/**`, `/lib64/**`, `/etc/nginx/**`, `/etc/ssl/**`, `/etc/hosts`, `/etc/resolv.conf`, `/etc/nsswitch.conf`, `/etc/passwd`, `/etc/group`, `/proc/**`
- Read access to project web root: `{{ project_root }}/current/{{ web_root }}/**`
- Read access to repo-local bones config: `{{ repo_path }}/bones/bones.yaml`, `{{ repo_path }}/bones/nginx.conf`
- Read/write/create access to socket directory: `/run/{{ project_name }}/** rwk`

**Denies:**
- `deny /root/** r`
- `deny /etc/ssh/** r`

**Intentional omission:**
- No blanket `/home/**` deny because default `repo_path` lives under `/home/{{ deploy_user }}/`
- Profile must read bones config from repo-local path

### 4. Playbook Orchestration

**Location:** `kit/remote/playbooks/setup.yml`

Shared orchestration:
```yaml
tasks:
  - include_role: users
  - include_role: apparmor    # Before nginx
  - include_role: common
  - include_role: runtime      # Optional per-template role from vars
  - include_role: nginx
  - include_role: ssl
```

AppArmor runs before nginx to ensure profile is loaded before service starts. Template-specific setup now comes from `templates/*/remote/vars/setup.yml` instead of duplicated playbooks.

### 5. bonesremote Doctor AppArmor Checks

**Location:** `crates/bonesremote/src/commands/doctor.rs`

New AppArmor validation checks:
- `check_apparmor_kernel_enabled` - reads `/sys/module/apparmor/parameters/enabled`
- `check_apparmor_service` - runs `systemctl is-active apparmor`
- `check_apparmor_profile_installed` - scans `/etc/apparmor.d/` for the project profile
- `check_apparmor_unit_wiring` - scans `/etc/systemd/system/*-nginx.service` for AppArmor wiring

Doctor now checks AppArmor before Landlock, reflecting the AppArmor-first policy.

### 6. Linux-Only Landlock Gating

**Location:** `crates/bonesremote/src/landlock.rs`

Fixed macOS clippy failure by removing the `allow(dead_code)` workaround and making the non-Linux stub explicitly use the policy fields:

```rust
fn policy_path_counts(policy: &Policy) -> (usize, usize) {
    (policy.read_only_paths.len(), policy.writable_paths.len())
}

#[cfg(not(target_os = "linux"))]
mod platform {
    pub fn restrict_self(policy: &Policy) -> Result<()> {
        let _ = super::policy_path_counts(policy);
        bail!("Landlock is only available on Linux")
    }
}
```

This ensures:
- No dead code warnings on non-Linux platforms
- Clear runtime error if Landlock APIs are called outside Linux
- Policy struct remains consistent across platforms

### 7. Template Overrides

**Locations:** `templates/*/remote/vars/setup.yml`

Framework templates now only override setup metadata such as runtime role name and completion label; the shared playbook owns the execution order.

## Documentation Updates

- `docs/commands/bonesdeploy/remote-setup.md` - Added AppArmor verification runbook for Linux hosts
- `docs/commands/bonesremote/doctor.md` - Documented AppArmor checks and updated summary table
- `docs/commands/bonesremote/landlock-nginx.md` - Updated to reflect repo-local nginx.conf as allowed read path
- `docs/security/05-apparmor-policy.md` - AppArmor-first policy intent
- `docs/security/19-agent-audit-checklist.md` - Added AppArmor service binding verification commands

## Known Gaps

### `--tags` Not Implemented for `remote setup`

The docs reference:
```bash
bonesdeploy remote setup --tags apparmor,nginx
```

This is **not currently implemented** in the CLI. The `remote setup` command does not accept arbitrary Ansible arguments. The only current interface is:
```bash
bonesdeploy remote setup
```

For selective provisioning, you must either:
1. Run the full playbook (current behavior)
2. Edit the playbook temporarily
3. Use `ansible-playbook` directly with the generated roles

This docs drift should be addressed in a follow-up by adding `--tags` support to `RemoteCommand::Setup` if selective runs are needed.

## Verification Steps

On a Linux host after `bonesdeploy remote setup`:

```bash
systemctl is-active apparmor
cat /sys/module/apparmor/parameters/enabled
grep '^profile bonesdeploy-<project>-nginx ' /etc/apparmor.d/bonesdeploy-<project>-nginx
systemctl cat <project>-nginx.service | grep -E 'AppArmorProfile|After=|Requires='
systemctl is-active <project>-nginx
```

Expected:
- `apparmor` service is `active`
- kernel parameter is `Y`/`yes`/`1`
- profile file exists and defines `bonesdeploy-<project>-nginx`
- service unit binds `AppArmorProfile=bonesdeploy-<project>-nginx` and declares `apparmor.service` in both `After=` and `Requires=`
- per-site nginx is `active`

## Design Rationale

### Why AppArmor-First

- Per-site nginx only needs to listen on unix sockets
- Router nginx handles external traffic
- AppArmor provides kernel-enforced confinement independent of application cooperation
- Simpler to reason about than Landlock for service boundaries

### Why Landlock Remains

- Supplemental job-time sandbox during deployment operations
- Defense-in-depth for bonesremote execution context
- Already implemented and tested on Linux hosts

### Why No `/home` Deny

- Default `repo_path` is `/home/{{ deploy_user }}/{{ project_name }}.git`
- Profile must read `bones.yaml` and `nginx.conf` from repo-local path
- Blanket `/home/**` deny would break default configuration
- Targeted denies (`/root/**`, `/etc/ssh/**`) cover sensitive paths without breaking defaults

## Files Changed

```
docs/goal.md
docs/commands/bonesdeploy/remote-setup.md
docs/commands/bonesremote/doctor.md
docs/commands/bonesremote/landlock-nginx.md
docs/security/05-apparmor-policy.md
docs/security/19-agent-audit-checklist.md
kit/remote/playbooks/setup.yml
kit/remote/nginx/site-nginx.service.j2
kit/remote/apparmor/project-nginx-profile.j2
kit/remote/roles/apparmor/tasks/main.yml
kit/remote/roles/apparmor/defaults/main.yml
kit/remote/roles/apparmor/handlers/main.yml
kit/remote/roles/apparmor/README.md
templates/*/remote/vars/setup.yml
crates/bonesdeploy/src/commands/init.rs
crates/bonesdeploy/src/config.rs
crates/bonesremote/src/landlock.rs
crates/bonesremote/src/commands/doctor.rs
crates/bonesremote/src/commands/landlock_nginx.rs
crates/bonesremote/src/commands/wire_release.rs
crates/bonesremote/Cargo.toml
```
