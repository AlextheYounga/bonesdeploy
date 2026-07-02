# Security Model Migration Plan Addendum

These decisions are now fixed:

- Service restart names stay on the existing `<project>-nginx.service` convention.
- Build containers do not mount any caches.
- Secret delivery remains user-managed through `bonesdeploy secrets push` for now.
- The secret target is always `shared/.env`.

This addendum only records the decisions above. It does not change the main migration plan.
