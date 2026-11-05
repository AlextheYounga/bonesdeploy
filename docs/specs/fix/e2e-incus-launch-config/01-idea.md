# Idea

## Request

Correct the E2E failure encountered while running all framework scenarios, and
record the error and its repair path in durable planning documentation.

## Problem

Every ignored E2E scenario fails before framework setup because
`Container::launch` passes a trailing `--config` argument to `incus launch`
without the required configuration value. Incus rejects the command with
`Error: flag needs an argument: --config`, so neither the cached base image nor
the shared test container can be created.

## Definitions

**Container launch configuration:** The complete sequence of `--config` option
and `key=value` argument pairs supplied by `Container::launch` to the Incus
CLI. It includes the memory, CPU, and nesting settings required by E2E
containers; it excludes a standalone option with no value.

## Desired outcome

`Container::launch` invokes `incus launch` with only complete configuration
pairs. With a reachable, initialized Incus daemon, the E2E harness can create
its base image and shared container, allowing every framework scenario to run.

## Scope

- Correct the Incus argument list owned by `e2e/src/container.rs`.
- Validate the corrected command through an E2E scenario and the complete
  serialized E2E framework matrix.

## Constraints

- Create the repair branch through Git Flow with a linked worktree.
- Preserve the existing memory, CPU, and nesting configuration values.
- Keep the E2E suite serialized because scenarios share the Incus daemon.
- Do not modify application framework fixtures or deployment behavior.

## Exclusions

- Starting, initializing, or granting access to the local Incus daemon is host
  setup, not an application-code fix.
- Changing Incus resource limits or nesting behavior is out of scope.
- Repairing failures that occur after container launch is out of scope for this
  bugfix and requires separate investigation.
