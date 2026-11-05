# Clarification

## Trigger

The decentralization implementation removed the existing update-patch system
and added a standalone `bonesdeploy migrate` command. This left `update`
calling a deleted Python command and made updates fail.

## Decision

Restore the Python-owned ordered update-patch registry. The `0.8.0` local patch
is the only transition from the old `.bones` workspace to project-local
`infra/`. It copies only validated project-owned content, preserves encrypted
secret bytes, verifies the copied tree before removing the old workspace, and
does not create a Git commit. It writes its local completion marker only after
the transition succeeds.

`bonesdeploy update` is the single deliberate bridge. The standalone
`bonesdeploy migrate` command is removed. Ordinary commands continue to reject
old `.bones` layouts and direct users to `bonesdeploy update`. Remote patch
execution remains part of the patch system but this layout-only patch performs
no obsolete config-repository operation remotely.

## Supersedes

Supersedes the standalone migration-tool decision in
`04-no-backwards-compat-clarity.md` and the corresponding migration-command
references in the authoritative record.

## Effect on the record

- `01-idea.md`: Names `bonesdeploy update` as the single versioned bridge.
- `02-plan.md`: Restores update-patch ownership and assigns the layout
  transformation to the `0.8.0` local patch.
- `03-tasks.md`: Replaces the completed standalone migration task with the
  completed patch-registry and update-transition task.
