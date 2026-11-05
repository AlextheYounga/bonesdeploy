# Plan

## Current behavior

`crates/bonesdeploy/src/cli/args.rs` gives setup commands short descriptions
that omit once-per-host and once-per-project scope. The combined setup in
`commands/setup.rs` always runs both phases. `commands/init/mod.rs` directs every
initialized project to `bonesdeploy server setup`, even when its host is already
prepared. Embedded skill documents contain pieces of the correct model, but do
not present a prominent additional-project workflow. README setup instructions
also omit explicit execution frequency.

## Intended behavior

Rendered help names the scope and frequency of each setup command. Initialization
recommends combined setup for a first project while explicitly directing projects
on prepared hosts to site setup. Agent and user documentation show both fresh-host
and additional-project workflows using consistent terminology.

## Approach

Revise existing descriptions rather than adding commands or detection logic.
Keep combined setup as the fresh-host convenience path. Add focused integration
tests that execute the binary and assert the scope wording in root, server, and
site help, plus the two initialization next steps.

## Responsibilities and boundaries

Clap declarations own command help. Initialization owns immediate next-step
guidance. Embedded skill assets own agent operating instructions. README owns
the human setup workflow. Integration tests validate visible CLI contracts.

## Affected areas

- `crates/bonesdeploy/src/cli/args.rs`
- `crates/bonesdeploy/src/commands/init/mod.rs`
- `crates/bonesdeploy/assets/skill/SKILL.md`
- `crates/bonesdeploy/assets/skill/commands.md`
- `crates/bonesdeploy/assets/skill/workflows.md`
- `crates/bonesdeploy/tests/cli.rs`
- `crates/bonesdeploy/tests/init.rs`
- `README.md`

## Decisions

Do not change setup execution semantics because the defect is guidance, not
orchestration. Recommend combined setup only for a project's first setup on a
fresh host. Recommend site setup directly for every additional project on an
already prepared host.

## Risks

Overly terse help could remain ambiguous, while overly detailed summaries could
make root help noisy. Exact-string tests could become brittle, so assertions
will target the essential scope phrases rather than entire help snapshots.

## Validation

Run focused BonesDeploy CLI and init tests and manually inspect rendered help.
Run `cargo fmt`, `cargo clippy`, and `shfmt -w .`. Review the final diff for
consistent scope terminology and unintended changes. Do not run e2e tests.
