# Git Deployment SSH Entry Point Plan

## Current behavior

The current branch contains an incomplete first pass. It opens the deployment
session as `git` and sudoes configuration sync, but invokes `bonesremote deploy`
without sudo even though the lifecycle requires root. It also moves snapshots,
state, and locks despite the selected design not requiring those migrations.

Before that first pass, BonesRemote owned the complete root-required lifecycle.
It serializes site mutations with the existing lock, stores deployment state
below the existing root-owned BonesRemote site root, controls the release
namespace, executes build scripts through the dedicated build identity, and
executes prepare scripts through the site runtime identity.

## Intended behavior

The local deploy command opens one SSH session as the standard deploy identity,
`git`. In that session it runs, in order:

```text
sudo -n bonesremote config sync --site <site>
sudo -n bonesremote deploy --site <site>
```

The first command receives the sanitized descriptor on stdin and stores
`/srv/conf/<site>/bones.json`. The second loads that snapshot and runs the
existing lifecycle as root. Sudoers accepts no other operation or argument
shape.

The change is specifically an SSH-entry-point restriction. It does not claim
that the `bonesremote deploy` process has effective UID `git`; sudo executes
that process as root. The security property is that routine deployment does not
open a root SSH session and that the `git` session can elevate only through two
exact command forms.

## Approach

Remove the unnecessary first-pass snapshot, state, lock, and lifecycle ownership
changes. Keep the deploy command's `git` SSH connection and route both operations
directly through sudo. Config sync owns descriptor stdin and deploy loads the
stored snapshot. Use anchored sudoers regexes, add allow and deny tests, and
preserve the provisioned control-plane directory mode. Do not introduce
lifecycle transition commands or move stateful data.

## Responsibilities and boundaries

- `bonesdeploy deploy` owns connection as `git`, descriptor serialization, and
  ordered invocation of the two sudo commands.
- `bonesremote config sync` always reads, validates, and installs the sanitized descriptor
  using the existing root-owned control-plane location.
- `bonesremote deploy` loads that control-plane snapshot and owns the existing
  root-required lifecycle without new orchestration boundaries.
- Existing build and prepare runners retain responsibility for dropping to
  `<site>-build` and `<site>` before repository scripts execute.
- BonesInfra owns rendering and validating the direct sudoers policy.

## Affected areas

- `crates/bonesdeploy/src/commands/deploy.rs`
- `crates/bonesdeploy/src/infra/mod.rs`
- `crates/bonesremote/src/cli/args.rs`
- `crates/bonesremote/src/commands/config.rs`
- `crates/bonesremote/src/commands/deploy/lifecycle.rs`
- `crates/bonesinfra/python/src/bonesinfra/assets/sudoers/bonesdeploy.j2`
- Focused Rust and Python tests for command construction and sudoers rendering
- Documentation describing deployment identities and trust boundaries
- The embedded BonesInfra wheel after the sudoers template changes

## Decisions

- Permit only fixed direct config-sync and deploy commands through sudo instead
  of creating a second lifecycle.
- Keep `bonesremote deploy` root-only because the current lifecycle legitimately
  owns root-controlled state, releases, activation, and service operations.
- Keep configuration sync as the sole stdin consumer and deploy as the sole
  loader of the synchronized descriptor.
- Keep all existing state and filesystem locations; no migration is required.
- Use `sudo -n` so deployment fails immediately rather than prompting when the
  server policy is missing or incorrect.

## Risks

- An overly broad sudoers rule could grant `git` unintended BonesRemote
  authority. Complete anchored command matching and denial tests mitigate this.
- A compromised shared `git` identity can deploy any site allowed by the site
  pattern. This is the existing documented single-operator model, not tenant
  isolation.
- Repository content is processed by a root-owned lifecycle. Existing identity
  boundaries must continue to ensure build and prepare scripts themselves never
  run as root.

## Validation

- Rust tests prove config sync reads stdin without a flag, deploy loads the
  stored snapshot, and local deploy constructs both direct sudo commands.
- Python tests pin the complete rendered sudoers allowlist.
- Denial tests cover arbitrary subcommands, omitted arguments, reordered
  arguments, optional deployment arguments, and trailing arguments.
- Existing lifecycle tests prove build scripts still use `<site>-build`, prepare
  scripts still use `<site>`, and transactional activation behavior is unchanged.
- Run `cargo test --workspace --exclude e2e`, `cargo clippy`, `cargo fmt`,
  `ruff check .`, `ruff format .`, `uv run pytest`, wheel regeneration, and
  `shfmt -w .`. Do not run E2E locally.
