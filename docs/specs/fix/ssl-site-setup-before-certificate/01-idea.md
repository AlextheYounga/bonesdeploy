# Idea

## Request

During manual testing, make site setup work before site SSL so the remaining test sites can be provisioned, certified, and deployed in that order.

## Problem

`bonesdeploy site setup --yes` fails nginx validation for a project whose configuration records SSL as enabled but whose Let's Encrypt certificate is absent. The router references the absent certificate, even though the documented workflow makes certificate acquisition a later `bonesdeploy site ssl` step.

## Definitions

**Certificate-ready:** Both required Let's Encrypt certificate files for the configured domain exist on the target server.

## Desired outcome

Site setup completes with an HTTP router when a certificate is absent, allowing the later SSL command to serve the ACME challenge and enable HTTPS. A certificate-ready site receives its HTTPS router during setup without relying on a local SSL flag.

## Scope

Certificate-derived SSL state across local configuration, provisioning, native nginx setup, manifests, next-step guidance, and focused tests.

## Constraints

Use the existing pyinfra remote facts and router rendering path. Preserve the existing SSL command as the owner of certificate acquisition and HTTPS activation. The fix is developed on the Git Flow bugfix branch requested for discovered defects.

## Exclusions

This change does not alter ACME issuance, certificate renewal, router templates, or the site SSL command's domain and email validation.
