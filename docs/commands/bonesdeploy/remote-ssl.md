# bonesdeploy remote ssl

## Overview

Configures SSL/TLS certificates for the deployment using Let's Encrypt and certbot. This command runs the SSL pyinfra deploy script (`.bones/infra/ssl.py`) as root, separate from the runtime deploy, keeping certificate management decoupled from app runtime concerns.

## Command Signature

```bash
bonesdeploy remote ssl [--domain <domain>] [--email <email>]
```

**Flags:**
- `--domain <domain>`: Domain name for the certificate (e.g., `app.example.com`)
- `--email <email>`: Email for Let's Encrypt registration and notices

Both flags are optional if values are already configured in `bones.yaml`.

---

## Architecture

The SSL command is fully separated from `remote runtime`:

- **`remote runtime`**: Configures framework services, AppArmor, nginx, and runs doctor — no SSL involvement
- **`remote ssl`**: Runs only the SSL deploy — handles certbot, challenges, and TLS configuration

This separation ensures:
1. Runtime updates don't accidentally trigger certificate operations
2. SSL can be run independently without full runtime reconfiguration
3. Certificate renewal doesn't require runtime deploy changes

---

## Detailed Execution Steps

### 1. Load Configuration

```rust
let bones_yaml = Path::new(config::Constants::BONES_YAML);
let mut cfg = config::load(bones_yaml)?;
```

Loads the existing configuration. Resolves `domain` and `email` from CLI args, existing config, or interactive prompts.

### 2. Validate Required Fields

Ensures both domain and email are set before proceeding. These are required for Let's Encrypt certificate issuance.

### 3. Ensure Runtime Assets Exist

Scaffolds `.bones/infra/` and `.bones/runtime/` base assets if missing, ensuring the SSL deploy script and the nginx router template exist.

### 4. Run pyinfra SSL Deploy (as Root)

Connects as root (or `BONES_BOOTSTRAP_SSH_USER`) since certbot and nginx configuration require elevated privileges:

```bash
pyinfra <host> .bones/infra/ssl.py --ssh-user root --ssh-port 22 --data ssl_domain=app.example.com --data ssl_email=ops@example.com --data paths.repo=/home/git/myapp.git ... -vv
```

**pyinfra Data Variables:**
- `ssl_domain` — domain for the certificate
- `ssl_email` — email for Let's Encrypt
- `nginx_ssl_certificate_path` — path to fullchain.pem
- `nginx_ssl_certificate_key_path` — path to privkey.pem
- `project_name`, `service_user`, `group` — deployment metadata
- `paths.*` — computed `DeploymentPaths` fields, flattened as dotted keys (e.g. `paths.repo`, `paths.current`)

### 5. SSL Deploy Tasks

The SSL pyinfra deploy performs these operations in order:

1. **Validate SSL inputs** — ensure domain and email are provided
2. **Render nginx HTTP challenge config** — temporary HTTP-only router config for certbot challenge
3. **Validate and reload nginx** — apply the HTTP challenge config
4. **Obtain certificate** — `certbot certonly` via webroot challenge
5. **Render nginx HTTPS config** — final router config with TLS enabled (port 443, HTTP→HTTPS redirect)
6. **Validate and reload nginx** — apply the HTTPS config

### 6. Sync Configuration

After SSL succeeds, saves the updated `bones.yaml` and syncs `.bones/` to the remote bare repo via `bonesdeploy push`.

---

## SSL Configuration in bones.yaml

**Before `remote ssl`:**
```yaml
ssl:
  enabled: false
  domain: ""
  email: ""
```

**After `remote ssl --domain app.example.com --email admin@example.com`:**
```yaml
ssl:
  enabled: true
  domain: app.example.com
  email: admin@example.com
```

---

## What Gets Created

### Certificates
```
/etc/letsencrypt/live/app.example.com/
├── cert.pem          # Domain certificate
├── chain.pem         # Intermediate certificate
├── fullchain.pem     # Full certificate chain (used by Nginx)
└── privkey.pem       # Private key (used by Nginx)
```

### Nginx Configuration
```
/etc/nginx/sites-available/myapp.conf
  - HTTP redirect to HTTPS
  - SSL certificate configuration
  - Modern SSL settings (TLS 1.2/1.3)
  - Strong cipher suites
```

---

## Prerequisites

1. **Domain DNS**: Domain must resolve to the server's IP address
2. **Port 80 accessible**: Required for Let's Encrypt HTTP-01 challenge
3. **Port 443 accessible**: Required for HTTPS traffic
4. **Runtime configured**: `remote runtime` must have been run first

---

## Common Workflow

### Add SSL After Initial Setup
```bash
bonesdeploy init
bonesdeploy remote setup
bonesdeploy remote runtime
bonesdeploy push
git push production master

# Later, add SSL
bonesdeploy remote ssl --domain app.example.com --email admin@example.com
bonesdeploy push
```

---

## Troubleshooting

### Certificate Issuance Fails

1. **Domain doesn't resolve**: Check DNS configuration
   ```bash
   dig app.example.com
   ```

2. **Port 80 blocked**: Check firewall
   ```bash
   sudo ufw status
   sudo ufw allow 80/tcp
   sudo ufw allow 443/tcp
   ```

3. **Nginx not running**: Start Nginx
   ```bash
   sudo systemctl start nginx
   ```

---

## Related Commands

- `bonesdeploy remote setup` - Machine/bootstrap provisioning
- `bonesdeploy remote runtime` - App runtime configuration
- `bonesdeploy init` - Initialize project configuration
- `bonesdeploy push` - Sync configuration to remote
