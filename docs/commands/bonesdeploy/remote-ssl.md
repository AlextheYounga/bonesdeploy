# bonesdeploy remote ssl

## Overview

Configures SSL/TLS certificates for the deployment using Let's Encrypt and certbot. This command runs the SSL playbook directly, separate from the runtime playbook, keeping certificate management decoupled from app runtime concerns.

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

The SSL command is now fully separated from `remote runtime`:

- **`remote runtime`**: Configures framework services, AppArmor, nginx, and runs doctor - no SSL involvement
- **`remote ssl`**: Runs only the SSL playbook - handles certbot, challenges, and TLS configuration

This separation ensures:
1. Runtime updates don't accidentally trigger certificate operations
2. SSL can be run independently without full runtime reconfiguration
3. Certificate renewal doesn't require runtime playbook changes

---

## Detailed Execution Steps

### 1. Load Configuration

**Source:** `remote_ssl.rs`

```rust
let bones_yaml = Path::new(config::Constants::BONES_YAML);
let mut cfg = config::load(bones_yaml)?;
```

Loads the existing configuration.

---

### 2. Update SSL Configuration from Flags

If `--domain` or `--email` flags are provided, updates the configuration with those values.

---

### 3. Validate Required Fields

Ensures both domain and email are set before proceeding. These are required for Let's Encrypt certificate issuance.

---

### 4. Ensure Runtime Assets Exist

**Source:** `remote_ssl.rs`

```rust
ensure_runtime_assets_exist()?;
```

Scaffolds `.bones/runtime/` base assets if missing, ensuring the SSL playbook exists.

---

### 5. Run SSL Playbook

**Source:** `remote_ssl.rs`

Runs `.bones/runtime/playbooks/ssl.yml` directly (not via tags on runtime playbook).

**Ansible Variables:**
- `ssl_domain={domain}`: Domain for certificate
- `ssl_email={email}`: Email for Let's Encrypt
- `nginx_ssl_certificate_path`: Path to fullchain.pem
- `nginx_ssl_certificate_key_path`: Path to privkey.pem

---

### 6. SSL Playbook Tasks

The SSL playbook performs:

1. **Validate SSL inputs** - Ensure domain and email are provided
2. **Render nginx HTTP challenge config** - Temporary HTTP-only config for certbot challenge
3. **Validate nginx** - `nginx -t`
4. **Reload nginx** - Apply HTTP challenge config
5. **Obtain certificate** - certbot webroot challenge
6. **Render nginx HTTPS config** - Final config with TLS enabled
7. **Validate nginx** - `nginx -t`
8. **Reload nginx** - Apply HTTPS config

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
/etc/nginx/sites-available/myapp
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
