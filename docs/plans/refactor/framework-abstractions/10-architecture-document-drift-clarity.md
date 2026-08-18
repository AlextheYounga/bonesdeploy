# Clarification

## Trigger

The architecture-document audit found precise post-decentralization drift that must
be preserved for the documentation child change.

## Decision

The architecture documentation child change will correct these documented facts:

- project configuration is root `.env`, not `.bones/bones.toml`;
- project infrastructure is `infra/`, including
  `infra/provision/{core,custom}` and `infra/secrets/`;
- the deployment unit is a committed repository revision;
- removed config-repository, import/receive, push, pull, and deploy-on-push flows
  are not current architecture;
- the Rust Framework contract includes centralized validation, defaults, build
  environment generation, and permission defaults;
- the Python framework contract uses materialized core/custom packages;
- the only current migration is `0003-project-infra` at version `0.8.0`, with
  remote marker behavior;
- `SiteMutation` has its actual constructors and validated-config location;
- deployment state is stored in `deployment.json`;
- local and remote doctor flows reflect current layout and command ownership;
- `RuntimeBackend`, `Runtime.permissions`, manifest socket artifacts, current
  language-runtime signatures, and current service registry behavior are documented.

The child change will update both `docs/ARCHITECTURE.md` and
`docs/architecture/reference.md`; the parent and other child plans will link to
those corrected concepts rather than preserving conflicting summaries.

## Supersedes

This adds the exact architecture-document corrections that were previously described
only as stale documentation.

## Effect on the record

- `02-plan.md` identifies the documentation drift as a concrete current behavior.
- `03-tasks.md` requires the architecture child plan to cover these exact sections
  and post-decentralization facts.
