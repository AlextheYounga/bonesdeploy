# Containment Error Clarification

## Trigger

Implementation review found that the deployment coordinator currently aborts
and cleans release state for every build error. The approved constraint requires
containment-verification failure to preserve active release state instead.

## Decision

Build-user termination failures use a distinct containment error. The deployment
coordinator returns that error without deleting the build context, failed
release, or active deployment record. Ordinary build failures, including a
successfully contained script timeout, keep the existing abort cleanup.

## Supersedes

This adds the error-propagation detail needed to enforce the existing
fail-closed constraint. It does not replace an earlier decision.

## Required Authoritative Updates

- `02-plan.md` assigns containment-error handling to the deployment coordinator
  and includes its source file in the affected areas.
- `03-tasks.md` includes the coordinator behavior and its regression validation.
