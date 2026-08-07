# RON

Every RON struct value must include the corresponding Rust struct name. This
applies to top-level and nested values.

Prefer `RuntimeDefaults(...)` and `PermissionRule(...)`; do not use anonymous
`(...)` struct values. A named object identifies its subject where it is
defined instead of requiring the reader to infer the type from its fields or
consumer.
