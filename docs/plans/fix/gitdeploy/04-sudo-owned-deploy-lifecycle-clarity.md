# Clarification

## Trigger

The earlier plan interpreted “deploy as git” to mean that the
`bonesremote deploy` process and its coordinator must have effective UID
`git`. That interpretation introduced a second lifecycle composed of privileged
transition subcommands, plus new state, lock, and snapshot ownership boundaries.
The requested behavior was clarified to mean that routine deployment must enter
the server through the `git` SSH identity, which then invokes a tightly
allowlisted root-required BonesRemote lifecycle through sudo.

## Decision

Keep the existing root-executed BonesRemote lifecycle intact. Change
`bonesdeploy deploy` to SSH as `git`, then invoke exactly these commands through
`sudo -n`:

```text
bonesremote config sync --site <validated-site> --config-stdin
bonesremote deploy --site <validated-site>
```

Sudoers must anchor each complete argument list and deny all other subcommands,
optional deploy arguments, reordered arguments, and trailing arguments.
Deployment state, locks, snapshots, backup credentials, release ownership, and
lifecycle sequencing stay in their existing root-owned locations. Build and
prepare scripts retain their existing `<site>-build` and `<site>` execution
identities.

This design removes root SSH from routine deployment without creating a second
deployment lifecycle or migrating unrelated state.

## Supersedes

This supersedes the earlier definition of the deploy coordinator as a
`bonesremote deploy` process running with effective UID `git`. It also
supersedes the planned typed lifecycle transitions, git-owned deployment state,
separate lock root, separate snapshot root, and legacy-state migration.

## Effect on the record

`01-idea.md` now defines the deployment SSH identity separately from the
effective identity of the privileged lifecycle and narrows scope to the two sudo
commands. `02-plan.md` now retains the existing lifecycle and root-owned state
while specifying exact command construction, sudoers policy, risks, and tests.
`03-tasks.md` now removes the first-pass state and lifecycle changes, implements
the two-command sudo boundary, and validates both allow and deny cases.
