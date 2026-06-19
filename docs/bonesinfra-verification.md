# Bonesinfra Verification Concern

`bonesdeploy` currently clones `bonesinfra` from its Git repository and uses that checkout as executable deployment infrastructure. Today, that fetch trusts whatever is currently at the repository tip.

## Concern

If `bonesinfra` is fetched by branch tip alone, then the code we run during `remote setup`, `remote runtime`, or `remote ssl` is whatever the remote repository serves at that moment. That creates a supply-chain trust boundary:

- a bad commit on `bonesinfra` could be pulled and executed
- a compromised maintainer machine could sign and push unwanted changes
- a compromised hosting account could expose unwanted branch-tip contents

This matters most because some of that infrastructure runs with elevated privileges.

## Deferred direction

We discussed verifying `bonesinfra` against a shipped public GPG key.

The rough idea:

1. Sign `bonesinfra` commits with a trusted GPG key.
2. Ship the corresponding public key with `bonesdeploy`.
3. After clone or update, verify the fetched commit against that key.
4. Refuse to run unverified infrastructure.

That approach is promising, but we are deliberately **not** implementing it in the current refactor.

## Why it is deferred

- The current cleanup pass is focused on dead code, duplication, wrapper collapse, and guardrail fixes.
- GPG verification introduces release-process and key-rotation decisions that deserve focused design.
- We want a separate decision on whether to verify only the fetched tip or a broader history.

## Questions for later

- Should we verify only the fetched commit, or the full reachable history we rely on?
- How do we rotate trusted keys without breaking old `bonesdeploy` installs?
- Should we support multiple trusted keys?
- Should verification happen only on fresh clone, or on every run that touches `bonesinfra`?
- Is GPG commit verification enough, or do we also want release tags or commit pinning?

## Current status

Deferred. Track this separately from the slop refactor.
