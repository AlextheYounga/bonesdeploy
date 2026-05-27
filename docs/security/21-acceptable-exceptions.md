# BonesDeploy Acceptable Exceptions

## Purpose

Exceptions are allowed when they are explicit, narrow, and documented.
They should never silently become the default operating model.

## Required Fields

Each exception should include:

```yaml
exception:
  service: <service-name>
  control: <policy-control-being-relaxed>
  reason: <why-this-is-required>
  compensating_controls:
    - <control>
  review_date: <date>
```

## Example

```yaml
exception:
  service: image-processor.service
  control: MemoryMax higher than default
  reason: Large image transformations need more memory
  compensating_controls:
    - Runs as dedicated user
    - AppArmor enforced
    - No sudo
    - PrivateTmp enabled
  review_date: 2026-08-01
```

## Findings

The agent or operator should flag:

- undocumented exceptions
- broad exceptions with no compensating controls
- expired exceptions that were never reviewed
