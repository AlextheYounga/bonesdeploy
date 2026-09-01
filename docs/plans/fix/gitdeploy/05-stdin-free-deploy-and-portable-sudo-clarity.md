# Clarification

## Trigger

The reviewed implementation retained `--config-stdin` on `bonesremote deploy`
even though the new two-command flow first stores the descriptor with config
sync. That mismatch makes deploy fail. The anchored sudoers regular expressions
also require sudo 1.9.10+, which is not an acceptable host requirement.

## Decision

Remove `--config-stdin` from both public subcommands. `bonesremote config sync
--site <site>` always reads its sanitized descriptor from stdin and atomically
stores `/srv/conf/<site>/bones.json`. `bonesremote deploy --site <site>` always
loads that site-derived snapshot and runs the existing root-owned lifecycle.

Replace version-dependent sudoers regular expressions with a root-owned,
non-writable wrapper. Sudoers permits the wrapper only; it validates one
operation (`config-sync` or `deploy`) and one site identifier, rejects all other
arguments, then executes the corresponding fixed BonesRemote argv. This retains
the two-operation elevation boundary on all supported sudo versions without
requiring a sudo version floor.

Control-plane sync preserves the provisioned `/srv/conf/<site>` mode of `0750`.

## Supersedes

This supersedes the `--config-stdin` argument in the config-sync sudo command,
the deploy lifecycle's stdin descriptor input, and the two anchored sudoers
regular expressions. It supersedes the `0755` control-plane directory mode
applied during config sync.

## Effect on the record

`01-idea.md` now defines the two fixed wrapper operations. `02-plan.md` now
assigns stdin ownership to config sync, snapshot loading to deploy, and argument
validation to the root-owned wrapper. `03-tasks.md` replaces the completed
regex/flag work with the required corrective tasks and validation.
