# Tasks

## Implementation

- [ ] Add the focused `bonesdeploy::build` command runner and register it from `build/mod.rs`; it must concurrently drain stdout and stderr, retain bounded tails, apply each operation's configured deadline, terminate and wait on timeout, and return command results suitable for Compose inventory.
- [ ] Route native Docker availability, image, probe, container, script, ownership, and removal commands through the runner while preserving the existing builder-image and mounted-path contract.
- [ ] Implement native script-timeout cleanup in the settled order: terminate the container workload, wait for it, restore source/cache ownership through a short-lived existing builder-image cleanup context, remove stale container state, and preserve primary plus cleanup errors.
- [ ] Refactor Compose validation, pull, build, image discovery, tagging, and saving to use the runner and include bounded command diagnostics on nonzero exit.
- [ ] Add Compose generated-tag lifecycle tracking so successfully created release tags are removed after archive save and on every failure path without removing unrelated image tags.
- [ ] Extend crate-root fake-Docker integration coverage through public build APIs for configured and unbounded operation deadlines, nonzero stderr diagnostics, child/workload cleanup, ownership restoration, and Compose tag cleanup.

## Validation

- [ ] Verify the native timeout test observes child termination and waiting before ownership restoration and container removal.
- [ ] Verify failure assertions retain the original operation error and include cleanup failure context when cleanup also fails.
- [ ] Verify both output streams remain drainable for verbose commands and retained diagnostics stay within the runner's configured bound.
- [ ] Verify Compose tags are absent after successful save and after failures occurring after tagging.
- [ ] Run the affected `bonesdeploy` tests and relevant non-e2e workspace tests; record the observed results in completion notes.
- [ ] Run `cargo fmt`, `cargo clippy`, and `shfmt -w .`; address all warnings and errors.

## Completion

- [ ] Review the final diff for scope compliance: only the planned implementation and crate-root test files changed, with no e2e work, scaffold text, or unrelated configuration changes.
- [ ] Confirm the three authoritative planning files still describe one coherent approach and that timeout `0` remains explicitly unbounded.

## Completion notes

Implementation has not started. Completion notes will record only material deviations, validation evidence, discoveries, and deliberately unfinished work after the implementation tasks are performed.
