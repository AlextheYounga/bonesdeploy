# BonesDeploy Agent Audit Checklist

## Purpose

An auditing agent should collect and compare at least the following host information before evaluating compliance.

## System Overview

```bash
uname -a
lsb_release -a || cat /etc/os-release
systemctl --version
mount
findmnt
```

## Users and Groups

```bash
getent passwd
getent group
sudo -l -U <service-user>
groups <service-user>
```

Check whether service users:

- have login shells
- belong to `sudo`, `wheel`, or `docker`
- own or can read unrelated project directories
- can read SSH keys or secrets

## Filesystem Layout

```bash
ls -lah /srv
ls -lah /srv/deployments
find /srv/deployments -maxdepth 3 -type d -printf '%m %u %g %p\n'
find /srv/deployments -name '.env' -o -name '*.sqlite' -o -name '*.db' -o -name '*.pem' -o -name '*.key'
```

Check permissions, ownership, and public exposure.

## Web Server Exposure

```bash
ss -tulpen
nginx -T 2>/dev/null || true
caddy validate 2>/dev/null || true
```

Check exposed ports, roots, proxy targets, and accidental public access.

## Systemd Hardening

```bash
systemctl list-units --type=service
systemctl cat <service>
systemctl show <service>
systemd-analyze security <service>
```

Compare each application service to the policy baseline.

## AppArmor

```bash
grep '^bonesdeploy-<project>-nginx (enforce)$' /sys/kernel/security/apparmor/profiles
cat /sys/module/apparmor/parameters/enabled 2>/dev/null
ls -lah /etc/apparmor.d
systemctl cat <service> | grep -E 'AppArmorProfile|After=|Requires='
```

Check whether application services are confined and enforcing.

## Capabilities

```bash
getpcaps <pid>
capsh --print
systemctl show <service> -p CapabilityBoundingSet -p AmbientCapabilities
```

Check for unnecessary capabilities.

## Cgroups

```bash
systemctl show <service> -p MemoryMax -p MemoryHigh -p CPUQuotaPerSecUSec -p TasksMax
systemd-cgls
systemd-cgtop
```

Check resource isolation.

## Docker and Containers

```bash
getent group docker
ls -l /var/run/docker.sock
docker ps --format '{{.Names}} {{.Image}} {{.Ports}}'
docker inspect <container>
```

Check socket exposure, capabilities, mounted volumes, users, and security options.

## Secrets Search

Search for likely secret exposure carefully:

```bash
find /srv/deployments -type f \( -name '.env' -o -name '*.pem' -o -name '*.key' -o -name '*credentials*' -o -name '*secret*' \) -printf '%m %u %g %p\n'
```

Do not print secret contents into logs or reports unless explicitly requested.
Report paths and permissions only.
