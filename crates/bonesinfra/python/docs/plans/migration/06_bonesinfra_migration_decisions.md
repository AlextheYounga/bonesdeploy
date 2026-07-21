# BonesInfra Migration Decisions Addendum

These decisions close the open policy points from the migration notes.

## Resolved Decisions

1. `runtime_user` and `runtime_group` are the only per-site identity fields for
   the v1 migration notes.
2. `release_group` is legacy terminology and should not appear in future v1
   planning docs.
3. `bonesinfra` provisions host prerequisites and installs the single sudoers
   drop-in during setup.
4. `bonesremote` defines the command contract, but does not install sudoers.
5. Podman is required and is installed by `bonesinfra` as part of host setup.
6. `/srv/sites/<project>` is `root:root` and functions as a boundary anchor, not
   as a collaboration directory.
7. Frameworks own creation of their writable leaf paths and files under
   `shared/`.
8. `bonesinfra` provisions the `shared/` parent and its permissions, but does not
   pre-create framework-specific writable leaves.
9. `runtime doctor` is removed from `bonesinfra`.

## Practical Meaning

The migration docs should now read as follows:

- `git` is ingress only.
- The site runtime identity is `runtime_user` / `runtime_group`.
- Host provisioning belongs to `bonesinfra`.
- Deployment lifecycle belongs to `bonesremote`.
- Framework/runtime code creates its own writable state inside `shared/`.

## Notes

- Older notes may still mention `release_group` or framework-specific shared
  leaves as implementation history; those references should not be treated as
  unresolved policy.
- Sudoers is a `bonesinfra` concern. `bonesinfra` should keep the single global
  drop-in narrow and should not create per-project sudoers files unless a
  concrete lifecycle requirement appears.
