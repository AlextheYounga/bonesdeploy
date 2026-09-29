# Tasks

## Implementation

- [ ] Remove the trailing standalone `--config` from `Container::launch` while
  preserving its memory, CPU, and nesting configuration pairs.

## Validation

- [ ] Run an ignored E2E setup scenario with a reachable Incus daemon; verify
  container launch progresses beyond Incus argument parsing.
- [ ] Run `cargo test-e2e`; verify Django, Compose, Laravel, Next server/static,
  Nuxt server/static, Rails, SvelteKit, and Vue pass serially.
- [ ] Run `cargo fmt`, `cargo clippy`, and `shfmt -w .` without warnings or
  formatting changes that require further correction.

## Completion

- [ ] Review the final diff to confirm the static launch argument repair and
  durable plan are the only changes.

## Completion notes

The full E2E matrix was first run on 2026-09-29. It initially stopped because
the local Incus daemon was inactive; after the host daemon was started, all ten
framework scenarios reached `Container::launch` and failed on the same invalid
trailing `--config` option. No implementation change has been made pending
human review and approval of this plan.
