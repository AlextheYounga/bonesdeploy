# Distribution Python Runtime Tasks

## Implementation

- [ ] Replace production CPython source compilation with distribution Python and
  native dependency build packages installed through APT.
- [ ] Remove managed Python runtime paths and the obsolete source installer.
- [ ] Remove local Django dependency outputs from artifacts and create the target
  virtualenv before installing production requirements during prepare.
- [ ] Drop `python_version` from new remote descriptors while reading and
  discarding descriptors that previously included it.

## Validation

- [ ] Update focused Python and Rust coverage for provisioning, AppArmor,
  artifacts, prepare ordering, and descriptor compatibility.
- [ ] Rebuild the embedded BonesInfra wheel and run all required non-E2E checks.

## Completion

- [ ] Update runtime and deployment documentation for distribution Python,
  target-side pip installation, and host-version compatibility.
- [ ] Review the final diff for stale managed-Python paths and unrelated changes.

## Completion Notes

Implementation is pending.
