# Clarification

## Trigger

Repository investigation after Acta planning showed that Laravel's native PHP-FPM deployment path does not render or attach an application AppArmor profile. The framework-local `app-profile.j2` is unused by `laravel/infra/runtime.py`.

## Decision

The queue worker will not attach an AppArmor profile. It will run with the existing systemd hardening conventions, the Laravel site runtime user and group, strict systemd filesystem protection, and explicit write access to shared Laravel storage, current-release bootstrap cache, and the site deployment log directory. This matches the actual Laravel runtime boundary without adding an unrelated profile subsystem.

## Supersedes

The clarification supersedes the `01-idea.md` scope item and `02-plan.md` approach and decision stating that the worker reuses an existing Laravel FPM AppArmor profile.

## Effect on the record

`01-idea.md` now defines systemd hardening and explicit writable paths as the worker's security boundary. `02-plan.md` now records that no Laravel application profile exists on this path and removes the profile reuse step. `03-tasks.md` now requires worker provisioning without an AppArmor profile and with explicit shared Laravel writable paths.
