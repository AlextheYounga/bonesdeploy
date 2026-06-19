I've audited the workspace. Below is a refactor plan that follows AGENTS.md (lazy: deletion-first, no unrequested abstraction, no new deps, fewest files, `ponytail:` for shortcuts, strict at trust boundaries). This plan also reflects the clarifications we made while discussing SSH, `bonesinfra`, and repo housekeeping. I'm explicitly **rejecting** the 5-layer `cli/app/domain/infra/ui` restructure proposed in `docs/refactor-code.md` — it adds folders and indirection for no behavior gain, which violates AGENTS.md.

## Clarified decisions

- **Keep `__trash__/`.** It is intentional personal workspace storage, not project slop.
- **Keep `openssh`.** We only need simple remote commands, but `openssh` still buys us cross-platform behavior and is acceptable.
- **Do not revisit `KnownHosts::Accept` right now.** Earlier concern noted, but current behavior is acceptable for this project.
- **Do not add `bonesinfra` GPG verification in this refactor.** The supply-chain concern is real, but it is deferred into `docs/bonesinfra-verification.md`.
- **Do not force a `port` schema change.** The practical issue is duplicated parsing, not whether the config stores a string or integer.
- **Proceed with the rest of the cleanup plan.** Dead code, duplication, wrapper collapse, path cleanup, and test guardrail fixes are still in scope.

## Audit summary

59 `.rs` files, ~6,440 lines. Concrete slop found:
- **~400 lines of dead code**: `bonesremote/commands/manage.rs` (344, not in `mod.rs`, references nonexistent `cfg.data`), `shared::config::{Permissions, PathOverride}` (never referenced).
- **Two cleancode guardrails are no-ops**: `no_literal_wrapped_fallback` + `no_manufactured_success` scan `tests/cleancode/src/` instead of `crates/` (they only pass against themselves).
- **Committed build artifacts**: `bin/bonesdeploy` + `bin/bonesremote` (5.5 MB) reproducible by `compile.sh`.
- **Two deferred trust discussions and one immediate trust gap**: SSH host behavior is accepted for now, `bonesinfra` verification is deferred into a separate document, and `which`-resolved paths written into sudoers still need canonicalization.
- **Pervasive duplication**: `port: String` re-parsed in 2 sites + passed raw in 6; rsync invocation triplicated; `remote_bones_toml` built 2 ways; `runtime.toml` parsed by 2 types in 2 crates; hand-rolled TOML parser in `update.rs` reinvents the `toml` dep; `prompts::confirm_with_lines` reinvents `inquire::Confirm` used in the same file; defaulting expression ×7 in prompts.
- **Over-abstraction**: `release/scripts.rs` builds a generic `DeploymentRun<Out,Err>` + `ConsoleTargets` + `SharedWriter` + `TeeWriter` layer purely so tests can inject a `Cursor`. `config::Constants` Java-ism. ~10 single-field/one-line wrapper fns/structs.
- **No `ponytail:` markers anywhere** despite AGENTS.md requirement, on: rsync stdout parsing, `prune_old_releases` O(n²) re-sort, exec-by-path-prefix heuristic, `home_dir` `/root` fallback, URL hand-parsers, hardcoded `DEFAULT_WEB_ROOT` in `release_state`.
- **`test_init_project.rs`** is a full fixture harness (TempDir + git subprocess + `Mutex`+`OnceLock` + `unsafe env::set_var`) where AGENTS.md asks for the smallest runnable check; `test_doctor.rs` is the on-spec counterexample sitting right next to it.

## Proposed plan — deletion-first, 6 passes, small boring commits

### Pass 1 — Pure deletions (zero behavior change, ~450 lines gone)
1. Delete `crates/bonesremote/src/commands/manage.rs` (dead).
2. Delete `Permissions` + `PathOverride` from `shared/src/config.rs`.
3. Delete `bin/bonesdeploy` + `bin/bonesremote`; add `bin/` to `.gitignore`.
4. Delete WHAT comments and doc-comments that restate test/function names (pervasive, ~40 sites).
5. Delete `docs/refactor-code.md` (its proposed restructure violates AGENTS.md and we're not doing it).

### Pass 2 — Collapse wrappers and the `Constants` Java-ism
6. Delete `config::Constants` in both crates; use `shared::paths::*` consts directly.
7. Delete one-line forwarders: `init_project::collect`, `init_project::collect_non_interactive` (cfg-test), `bonesinfra::checkout_path`, `remote_data::setup`, `config::default_project_root_for`, `release_state::{deployment_paths, build_root, shared_dir, current_link}`, `wire_release::{shared_path_exists, release_path_is_resolved}` (identical dup).
8. Delete single-use wrapper structs: `InitOutcome` (return `bool`), `NonInteractiveValues`, `PullTarget` (use existing `git::RemoteConnectionDetails`).
9. Delete `release/scripts.rs` generic abstraction; `run_deployment_script` writes directly to `io::stdout()`/`io::stderr()`; rewrite the 4 tests as plain asserts against captured output (no `Cursor`/`Arc<Mutex>`/`TeeWriter`).
10. Delete `prompts::confirm_with_lines`; use `inquire::Confirm` (already a dep, already used in the file).
11. Delete `main.rs` `pub(crate) use infra::{...}` / `use ui::prompts` re-exports; callers use real paths.

### Pass 3 — De-duplicate
12. Keep `Bones.port` compatible as-is and add one shared parse helper; delete the duplicated `ssh.rs:10` + `update_release.rs:56` parse logic.
13. Add one `infra::rsync` helper; rewrite `push_state`, `pull_state`, `doctor::check_rsync_sync` to use it.
14. Standardize `remote_bones_toml` via `cfg.deployment_paths(...).repo_bones_toml` everywhere (rollback, manage-via-deploy, deploy).
15. Delete `wire_release::load_runtime_shared_paths` + local `RuntimeShared`; use `shared::config::load_runtime`.
16. Replace `update::parse_package_version` hand-parser with the `toml` crate (already installed).
17. Collapse `cli_or_prompt` / `cli_existing_or_prompt` into one fn.
18. Collapse the 7-copy `existing.map(...).filter(!empty).unwrap_or(default)` defaulting into one helper.
19. `git::list_remotes_with_urls` → parse `git remote -v` once (kills N+1 spawns).
20. Delete the 3 near-duplicated test config builders and 7 temp_dir helpers; one tiny helper per crate.

### Pass 4 — Trust boundaries (AGENTS.md: not lazy here)
21. Leave SSH trust behavior alone for now. Keep `openssh` and current `KnownHosts::Accept` behavior.
22. `bonesremote::init::which_bonesdeploy_remote`: canonicalize the resolved path before writing it into `/etc/sudoers.d/bonesdeploy`; reject if not under a sane root (`/usr/local/bin`, `/usr/bin`). `ponytail:` if we keep an allowlist.
23. Document the `bonesinfra` verification concern separately and defer implementation. See `docs/bonesinfra-verification.md`.
24. Validate `host` (hostname charset) at config load / remote-inference time, before it flows into any shell string.

### Pass 5 — `ponytail:` markers for kept shortcuts
25. Add `ponytail:` comments naming the ceiling + upgrade path for: `doctor::check_rsync_sync` stdout parsing, `post_deploy::prune_old_releases` O(n²) re-sort, `embedded::write_asset` exec-by-path-prefix, `python::runtime_defaults` hard-coded internal path, `paths::home_dir` `/root` fallback, `git::parse_ssh_style_url`/`parse_scp_style_url` (no IPv6 / percent-encoding), `release_state::deployment_paths` hardcoded `DEFAULT_WEB_ROOT`.

### Pass 6 — Fix the cleancode guardrails (so they actually enforce)
26. Fix `no_literal_wrapped_fallback` + `no_manufactured_success` to scan `crates/` (currently scanning themselves).
27. Extract one shared `collect_source_files` walker; unify skip lists (add `__trash__`, `.worktrees`).
28. Reconsider `no_legacy_terms` substring blacklist — either drop it or scope to identifiers only (it currently flags honest doc comments).
29. Add a guardrail: `shared::paths` must not grow past N lines / `Deployment` field count (prevents regression).
30. Strip `test_init_project.rs` harness for the pure-function tests; keep one minimal integration check.

## What I'm explicitly NOT doing (and why)
- **No 5-layer `cli/app/domain/infra/ui` restructure.** AGENTS.md: "No abstractions that weren't explicitly requested... Deletion over addition... Fewest files possible." The current `cli/`+`commands/`+`infra/`+`ui/` shape is fine once the slop is gone. `docs/refactor-code.md` proposes the opposite philosophy and should be deleted.
- **No adapter traits** for git/ssh/rsync/process. Folder boundary is enough.
- **No splitting `shared::paths`** into a `paths/` module tree. 270 lines, under the 400-line guardrail.
- **No new dependencies.** Everything above uses stdlib or already-installed crates (`toml`, `inquire`, `openssh`, `rust-embed`, `syn`).

## Order of risk
Pass 1 is pure deletion (safe). Pass 2 is mostly mechanical. Pass 3 stays lower risk now that `port` compatibility is preserved instead of forcing a schema change. Pass 4 is still security-sensitive, but SSH policy changes are explicitly out of scope for this pass. Pass 6 changes test behavior.
