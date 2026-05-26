# BonesDeploy Acceptable Exceptions

## Purpose

Exceptions are allowed when they are narrow, explicit, and documented.

## Rules

- Every exception should name the service, the control being relaxed, the reason, and the review date.
- Compensating controls should be listed alongside the exception.
- Exceptions should never become silent defaults.

## BonesDeploy Notes

- Use exceptions only when the current `deploy_user` / `service_user` / `public_path` model truly needs them.
- If an exception weakens the active release hardening, document the reason and the compensating controls.

## Example

```yaml
exception:
  service: example.service
  control: MemoryMax higher than default
  reason: Large workload needs more memory
  compensating_controls:
    - Dedicated service user
    - AppArmor enforced
    - No sudo
  review_date: 2026-08-01
```
