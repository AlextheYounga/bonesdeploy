# Deploy Account Name Clarity

## Trigger

The requested production deploy account name changed from `bonesdeploy` to
`deploy` after the initial implementation was completed.

## Decision

Use `deploy` as the global production deploy Unix account. Preserve the
product name `BonesDeploy` and the `/etc/sudoers.d/bonesdeploy` policy filename.
Reserve the project/site name `deploy` in both Rust and Python validation so a
runtime user and group cannot collide with the global deploy account.

The change remains a breaking fresh-host contract. Existing hosts using either
`git` or `bonesdeploy` must be reprovisioned; no account migration or SSH
fallback is added.

## Supersedes

Supersedes the account-name decision in `01-idea.md` and `02-plan.md` that
selected `bonesdeploy` as the Unix deploy account. The product and sudoers
policy names remain unchanged.

## Required Authoritative Updates

- `01-idea.md`: describe `deploy` as the deploy identity and reserve the
  project/site name.
- `02-plan.md`: update the intended behavior, decisions, risks, and validation
  to cover the account collision.
- `03-tasks.md`: add the account rename and project-name validation work.
