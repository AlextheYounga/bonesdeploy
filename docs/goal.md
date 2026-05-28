Goal
- Ensure this branch implements a per-project AppArmor-first setup per security docs, avoid Landlock where AppArmor should handle concerns, and make bonesremote doctor validate both AppArmor and Landlock correctly.

Constraints & Preferences
- Read and follow ./docs/PROJECT.md, ./docs/security/*, and ./docs/commands/*.
- In-scope: AppArmor implementation/policy alignment and reducing inappropriate Landlock usage.
- Out-of-scope: improving Landlock itself.
- User decision: doctor must enforce/check both AppArmor-first and Landlock.

We will use Ansible to initially set up AppArmor for the project using the `remote setup` command. 

Progress
Done
- Read docs/PROJECT.md.
- Reviewed security docs including:
- docs/security/05-apparmor-policy.md
- docs/security/06-landlock-policy.md
- docs/security/19-agent-audit-checklist.md
- docs/security/22-desired-end-state-summary.md
- Enumerated command docs under docs/commands/bonesdeploy/* and docs/commands/bonesremote/*.
- Identified drift/conflict: command docs still show primary runtime Landlock paths conflicting with AppArmor-first intent.
- Implemented AppArmor checks in crates/bonesremote/src/commands/doctor.rs:
- check_apparmor_support
- check_apparmor_kernel_enabled (/sys/module/apparmor/parameters/enabled)
- check_apparmor_service (systemctl is-active apparmor)
- check_apparmor_profiles_enforcing (aa-status)
- parsing helpers: apparmor_kernel_enabled, aa_status_has_enforcing_profiles
- Kept existing Landlock validation in doctor: check_landlock_support via landlock::verify_support().
- Added unit tests in crates/bonesremote/src/commands/doctor.rs for AppArmor parsing logic.
- Updated docs: docs/commands/bonesremote/doctor.md now documents AppArmor + Landlock checks and updated summary table.
- Ran cargo fmt successfully.
In Progress
- Linux host validation of the implemented AppArmor Ansible flow and service runtime behavior.
Blocked
- `cargo test -p bonesdeploy` and `cargo clippy -p bonesdeploy --all-targets` pass locally.
- `cargo test -p bonesremote` now compiles locally on macOS after target-gating Landlock, but Linux host validation is still required for real Landlock/AppArmor runtime behavior.
- Workspace-wide `cargo clippy --all-targets` still has unrelated preexisting lint failures in `bonesremote` test/helper code outside the AppArmor work.
Key Decisions
- bonesremote doctor should perform AppArmor-first validation and also Landlock validation (user-confirmed requirement).
- Missing/incorrect AppArmor should be reported as doctor issues, not optional warning.
Next Steps
- Run Linux validation:
1. `bonesdeploy remote setup` against a Linux host.
2. Verify `systemctl is-active apparmor` and `/sys/module/apparmor/parameters/enabled`.
3. Verify `aa-status` shows `bonesdeploy-<project>-nginx` in the enforce-mode section.
4. Verify `<project>-nginx.service` contains `AppArmorProfile=`, `After=... apparmor.service`, and `Requires=apparmor.service`.
5. Verify `systemctl is-active <project>-nginx`.
6. Run `cargo test -p bonesremote` and `cargo clippy -p bonesremote --all-targets` on Linux.
Critical Context
- doctor now includes AppArmor checks before Landlock checks.
- aa-status enforcement parsing now requires positive enforce count (rejects "0 profiles are in enforce mode.").
- AppArmor Ansible provisioning now exists in `kit/remote/roles/apparmor` and is wired into `kit/remote/playbooks/setup.yml` and template playbooks.
- Per-site systemd unit now binds AppArmor profile and requires startup ordering with `apparmor.service`.
- Command docs now include Linux verification runbook for AppArmor setup.
Relevant Files
- /Users/alexyounger/Development/Code/Rust/bonesdeploy/.worktrees/feat/apparmor/docs/PROJECT.md: project model, just-in-time mutation/security posture.
- /Users/alexyounger/Development/Code/Rust/bonesdeploy/.worktrees/feat/apparmor/docs/security/05-apparmor-policy.md: AppArmor-first policy intent.
- /Users/alexyounger/Development/Code/Rust/bonesdeploy/.worktrees/feat/apparmor/docs/security/06-landlock-policy.md: Landlock as supplemental job-time sandbox.
- /Users/alexyounger/Development/Code/Rust/bonesdeploy/.worktrees/feat/apparmor/docs/security/19-agent-audit-checklist.md: audit commands/checks for AppArmor and system hardening.
- /Users/alexyounger/Development/Code/Rust/bonesdeploy/.worktrees/feat/apparmor/docs/security/22-desired-end-state-summary.md: desired end-state includes AppArmor enforcing + Landlock where practical.
- /Users/alexyounger/Development/Code/Rust/bonesdeploy/.worktrees/feat/apparmor/crates/bonesremote/src/commands/doctor.rs: implemented AppArmor + Landlock doctor checks and unit tests.
- /Users/alexyounger/Development/Code/Rust/bonesdeploy/.worktrees/feat/apparmor/docs/commands/bonesremote/doctor.md: updated doctor command documentation.
- /Users/alexyounger/Development/Code/Rust/bonesdeploy/.worktrees/feat/apparmor/crates/bonesremote/src/landlock.rs: Landlock support verification and restriction code.
- /Users/alexyounger/Development/Code/Rust/bonesdeploy/.worktrees/feat/apparmor/kit/remote/nginx/site-nginx.service.j2: currently uses bonesremote landlock nginx in ExecStart.
- /Users/alexyounger/Development/Code/Rust/bonesdeploy/.worktrees/feat/apparmor/kit/remote/roles/nginx/tasks/main.yml: nginx/systemd provisioning path; no AppArmor provisioning observed.
- /Users/alexyounger/Development/Code/Rust/bonesdeploy/.worktrees/feat/apparmor/kit/remote/playbooks/setup.yml: role orchestration; no dedicated AppArmor role observed.
