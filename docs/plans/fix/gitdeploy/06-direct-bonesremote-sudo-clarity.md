# Clarification

## Trigger

The wrapper introduced to avoid a sudo version floor was rejected. Older sudo
versions are not a supported constraint for this change.

## Decision

Remove the wrapper completely. Routine deploy SSH invokes these direct commands:

```text
sudo -n bonesremote config sync --site <site>
sudo -n bonesremote deploy --site <site>
```

Sudoers uses anchored argument regular expressions for the installed absolute
BonesRemote path. The deployment environment requires sudo 1.9.10 or newer.
Config sync remains the sole stdin consumer and deploy remains the snapshot
loader. The control-plane directory remains mode `0750`.

## Supersedes

This supersedes the wrapper and portable-sudo policy in clarification 05.

## Effect on the record

`01-idea.md`, `02-plan.md`, and `03-tasks.md` restore direct BonesRemote sudo
commands and anchored policy tests, remove wrapper work, and record the sudo
1.9.10 requirement.
