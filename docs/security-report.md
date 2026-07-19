# BonesDeploy / BonesInfra Security Report

**Scope:** Static review of the current `release/0.7.1` sources and the
project's documented security model. No live host was tested, and no code was
changed while producing this report.

## Executive summary

The deployment and runtime isolation model is generally strong: projects have
separate runtime identities, build users, release trees, systemd units,
AppArmor profiles, and resource limits. The meaningful risks are concentrated
at the control-plane boundaries rather than in the application sandbox.

The following items are worth implementing. Their severity assumes that the
deployment host may contain more than one project or that project state may be
edited by someone less trusted than the host administrator.

| Priority | Issue | Location |
|---|---|---|
| High | SSH host identity is not authenticated consistently | `crates/bonesdeploy/src/infra/ssh.rs:27-38` |
| High, conditional | Imported configuration can direct a root process to write a Git hook outside the configured repository area | `crates/bonesremote/src/commands/site.rs:59-72` |
| Medium, conditional | One shared `git` identity can invoke allowed root commands for any site | `bonesinfra/src/bonesinfra/assets/sudoers/bonesdeploy.j2:2` |
| Medium | Failed secret uploads can leave plaintext temporary files | `crates/bonesdeploy/src/commands/secrets.rs:141-145` |
| Medium, operational | Restart reports success without confirming the services remain active | `crates/bonesremote/src/commands/service.rs:22-31` |

## Findings

### SEC-001 — SSH host-key verification can be bypassed

`connect_as` uses `KnownHosts::Accept`, which accepts changed host keys, and
`external_command` explicitly sets `StrictHostKeyChecking=no` and discards the
known-hosts file. The latter path is used by `push_state`, while the former is
used by the other remote operations.

This permits server impersonation and interception of deployment traffic. With
ordinary public-key authentication, an attacker cannot normally modify a
bundle in transit and then inject it into the genuine server; the more direct
risks are disclosure of the bundle, command redirection, false success, and
credential theft if interactive authentication is ever enabled.

**Remediation:** use one SSH implementation and require strict verification
after an explicit first-key enrollment. At minimum, use `accept-new` instead of
accepting changed keys, and never use `/dev/null` as the known-hosts file.

### SEC-002 — Imported `repo_path` is trusted by a root hook writer

`site import` validates the project name and rejects symlinks in the imported
dataset, but `write_post_receive_hook` still takes `repo_path` from the newly
imported `bones.toml` and writes `<repo_path>/hooks/post-receive` as root.

If a less-trusted contributor can cause an operator to publish project state,
this is a confused-deputy filesystem write. The fixed `hooks/post-receive`
suffix limits the impact, but the path can still target an unintended
repository or create directories outside the normal repository parent.

**Remediation:** resolve the configured repository parent on the host and
reject any imported path that is not the expected site repository beneath that
parent. Validate the canonical path before creating the hook.

### SEC-003 — Shared deploy identity has cross-site sudo authority

The sudoers rule grants the common deploy user `git` passwordless access to
several `bonesremote` commands with `--site *`. On a multi-tenant host, any
compromise of that identity can restart, roll back, prune, or trigger a hook
for another project.

This is not a tenant-isolation issue when one operator intentionally owns all
projects on a host. If tenant isolation is required, replacing `*` with
literal site names is insufficient by itself while all projects still share
the same Unix account and authorized keys.

**Remediation:** either document the single-operator trust model, or use
separate deploy identities/keys (or a root wrapper that derives the permitted
site from the authenticated key and repository) before attempting per-site
sudo rules.

### SEC-004 — Plaintext secret temporary file is not cleaned on failure

`secrets push` decrypts locally, streams the plaintext into a remote `mktemp`
file, and moves it into place. If `chown`, `chmod`, or `mv` fails, the
temporary file can remain on the server. `mktemp` normally creates a private
file, so this is residue risk rather than an immediate world-readable secret,
but it is still avoidable secret exposure.

**Remediation:** install an EXIT trap in the remote shell command that removes
the temporary path, and clear the variable after a successful move.

### SEC-005 — Service restart does not verify steady-state health

`bonesremote service restart` checks only the exit status of `systemctl
restart`. A unit can start successfully and then exit shortly afterward while
the command still reports `Restarted`.

This is primarily a deployment correctness and observability defect. It can
leave automation believing a rollback or deployment is healthy when the site
is not serving traffic.

**Remediation:** after restarting the target, verify that each required service
is active and remains active long enough to catch immediate failures. Return a
failure with the affected unit names and relevant journal output.

## Items intentionally excluded

The following are not retained as findings because they are either policy
choices, host-operator responsibilities, compatibility-sensitive defaults, or
documentation mismatches rather than demonstrated product defects:

- Generic sysctl, PAM, GRUB, Secure Boot, auditd/AIDE, journald, fail2ban, and
  unattended-upgrade baselines.
- Generic nginx CSP/security-header recommendations and TLS cipher policy.
- Blanket systemd restrictions such as `MemoryDenyWriteExecute` or broad
  syscall filters without runtime-specific testing.
- `--cap-drop=all`, read-only build containers, and network isolation without
  first defining which build features require package/network access.
- The unencrypted local GPG key, which is an explicit single-user workstation
  trade-off unless that threat model changes.
- Release-tree setgid propagation, which conflicts with a documentation claim
  but does not currently break access because ownership is explicitly set.
- `release kill` sudo coverage, because the user-facing command uses the
  configured privileged SSH connection.
- Extra `newrev` and negative-sudo checks, which are useful defense in depth but
  do not represent an independent release-blocking vulnerability.

## Recommended order

1. Fix host-key verification on every SSH path.
2. Constrain imported repository paths.
3. Add guaranteed cleanup for secret upload temporaries.
4. Decide whether hosts are single-operator or multi-tenant; redesign the
   deploy identity if tenant isolation is required.
5. Add post-restart steady-state verification.

This report is intentionally limited to issues with a clear owner, impact, and
reasonable implementation path. It should not be read as a claim that every
Lynis suggestion or Linux-hardening checklist item is required for this
project.
