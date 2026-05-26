# BonesDeploy Desired End State Summary

## Purpose

This section summarizes the target hardened state BonesDeploy should reach on a well-configured server.

## Summary

- one Unix service user per project
- deploy user separate from service user
- active release tree becomes service-owned after activation
- `public_path` exposes only the public surface
- secrets are not web-accessible
- systemd hardening is enabled per service
- AppArmor is enforcing where supported
- Landlock is used for risky child jobs where practical
- cgroup limits are set
- dangerous capabilities are dropped by default
- no Docker socket exposure to apps

## BonesDeploy Principle

A compromised app should be able to damage only itself and the small set of resources it explicitly needs.
