# BonesDeploy Package Manager and Runtime Policy

## Purpose

Production services should not generally need package-manager privileges or write access to dependency stores at runtime.

## Package Managers in Production

The agent or operator should flag if service users can run or write to package-manager global locations unnecessarily:

```text
npm global dirs
composer global auth or cache
pip global locations
gem global paths
cargo global paths
```

## Build vs Runtime Separation

Dependencies should be installed during build or deploy, not by the runtime web process.
Service users should not need write access to:

```text
node_modules
vendor
.venv
bundle
```

unless the application is explicitly designed for dynamic plugin or dependency installation.

## Findings

The agent or operator should flag:

- service user can write package-manager globals
- runtime process installs dependencies on demand
- dependency directories are writable when they should not be
