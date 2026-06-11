# bonesdeploy remote ssl

## Overview

Configures SSL/TLS certificates for the deployment using Let's Encrypt and certbot. This command runs the remote setup playbook with SSL-specific variables, obtaining certificates and updating the runtime nginx router for HTTPS. It updates the `bones.yaml` configuration to mark SSL as enabled after successful setup.

## Command Signature

```bash
bonesdeploy remote ssl [--domain <domain>] [--email <email>]
```

**Flags:**
- `--domain <domain>`: Domain name for the certificate (e.g., `app.example.com`)
- `--email <email>`: Email for Let's Encrypt registration and notices

Both flags are optional if values are already configured in `bones.yaml`.

---

## Detailed Execution Steps

### 1. Load Configuration

**Source:** `remote_ssl.rs:10-11`

```rust
let bones_yaml = Path::new(config::Constants::BONES_YAML);
let mut cfg = config::load(bones_yaml)?;
```

Loads the existing configuration. Uses `mut` because SSL settings will be updated.

---

### 2. Update SSL Configuration from Flags

**Source:** `remote_ssl.rs:13-19`

```rust
if let Some(value) = domain {
    cfg.ssl.domain = value;
}

if let Some(value) = email {
    cfg.ssl.email = value;
}
```

If `--domain` or `--email` flags are provided, updates the configuration with those values. This allows overriding or setting SSL parameters without editing `bones.yaml` manually.

---

### 3. Validate Required Fields

**Source:** `remote_ssl.rs:21-27`

```rust
if cfg.ssl.domain.is_empty() {
    bail!("SSL domain is missing. Pass --domain or set ssl.domain in .bones/bones.yaml");
}

if cfg.ssl.email.is_empty() {
    bail!("SSL email is missing. Pass --email or set ssl.email in .bones/bones.yaml");
}
```

Ensures both domain and email are set before proceeding. These are required for Let's Encrypt certificate issuance.

**Domain:** The domain name the certificate will be issued for (must resolve to the server)
**Email:** Used for Let's Encrypt account registration and expiration notices

---

### 4. Save Updated Configuration

**Source:** `remote_ssl.rs:29`

```rust
config::save(&cfg, bones_yaml)?;
```

Saves the configuration with updated SSL settings. This ensures the domain and email persist across commands.

---

### 5. Ensure Ansible is Installed

**Source:** `remote_ssl.rs:31`

```rust
remote_setup::ensure_ansible_playbook_installed()?;
```

Verifies Ansible is available, auto-installing if necessary (same logic as `bonesdeploy remote setup`).

---

### 6. Print SSL Setup Header

**Source:** `remote_ssl.rs:33-38`

```rust
println!(
    "Running {} against {} for {}...",
    style("remote ssl").cyan().bold(),
    style(&cfg.data.host).cyan(),
    style(&cfg.ssl.domain).cyan(),
);
```

Displays the target host and domain being configured.

---

### 7. Construct SSL-Specific Ansible Variables

**Source:** `remote_ssl.rs:40-49`

```rust
let extra_args = vec![
    String::from("--tags"),
        String::from("ssl"),
    String::from("-e"),
    String::from("ssl_enabled=true"),
    String::from("-e"),
    format!("ssl_domain={}", cfg.ssl.domain),
    String::from("-e"),
    format!("ssl_email={}", cfg.ssl.email),
];
```

**Ansible Tags:**
- `--tags ssl`: Only runs SSL tasks in the shared remote setup playbook

**Extra Variables:**
- `ssl_enabled=true`: Signals SSL should be configured
- `ssl_domain={domain}`: Domain for certificate
- `ssl_email={email}`: Email for Let's Encrypt

**Why tags?** Running only tagged tasks makes this command idempotent and faster:
- Doesn't re-run user creation, directory setup, etc.
- Focuses on Nginx configuration and certificate installation
- Can be run multiple times without side effects

---

### 8. Disable SSL Flag for Ansible Run

**Source:** `remote_ssl.rs:51-52`

```rust
let mut cfg_for_run = cfg.clone();
cfg_for_run.ssl.enabled = false;
```

Temporarily disables the `ssl.enabled` flag for the Ansible run. This prevents the playbook from using existing SSL configuration during setup.

**Why?** The Ansible playbook needs the runtime router and certbot pieces in place before it can validate the domain and switch to HTTPS.

---

### 9. Run Ansible Playbook

**Source:** `remote_ssl.rs:54`

```rust
remote_setup::run_ansible_playbook(&cfg_for_run, &cfg.permissions.defaults.deploy_user, &extra_args)?;
```

Executes the remote setup playbook with SSL-specific variables and tags.

#### 9.1 Playbook SSL Tasks

The Ansible playbook typically performs:

1. **Install certbot**
   ```yaml
   - name: Install certbot
     apt:
       name: certbot
       state: present
   ```

2. **Install certbot Nginx plugin**
   ```yaml
   - name: Install python3-certbot-nginx
     apt:
       name: python3-certbot-nginx
       state: present
   ```

3. **Obtain SSL certificate**
   ```yaml
   - name: Obtain SSL certificate
     command: >
       certbot certonly --nginx
       --non-interactive
       --agree-tos
       --email {{ ssl_email }}
       -d {{ ssl_domain }}
     args:
       creates: "/etc/letsencrypt/live/{{ ssl_domain }}/fullchain.pem"
   ```

4. **Configure Nginx for HTTPS**
   ```yaml
   - name: Configure Nginx with SSL
     template:
       src: nginx-ssl.conf.j2
       dest: "/etc/nginx/sites-available/{{ project_name }}"
     notify: reload nginx
   ```

5. **Set up auto-renewal**
   ```yaml
   - name: Enable certbot auto-renewal
     systemd:
       name: certbot.timer
       enabled: yes
       state: started
   ```

---

### 10. Enable SSL Flag in Configuration

**Source:** `remote_ssl.rs:56-57`

```rust
cfg.ssl.enabled = true;
config::save(&cfg, bones_yaml)?;
```

After successful SSL setup, marks SSL as enabled in `bones.yaml`. This signals to future `remote setup` runs and deployment processes that SSL should be used.

---

### 11. Print Success Message

**Source:** `remote_ssl.rs:59`

```rust
println!("\n{} SSL setup complete.", style("Done!").green().bold());
```

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

### Auto-Renewal
```
systemd:
  certbot.timer - Runs certbot renew twice daily
  certbot.service - Executed by timer to check/renew certificates
```

---

## Prerequisites

1. **Domain DNS**: Domain must resolve to the server's IP address
2. **Port 80 accessible**: Required for Let's Encrypt HTTP-01 challenge
3. **Port 443 accessible**: Required for HTTPS traffic
4. **Nginx installed**: Installed during initial `remote setup`

---

## Common Workflow

### Initial Setup (No SSL)
```bash
bonesdeploy init
bonesdeploy remote setup
bonesdeploy push
git push production master
```

### Add SSL Later
```bash
# Configure SSL
bonesdeploy remote ssl --domain app.example.com --email admin@example.com

# Push updated configuration (now with ssl.enabled=true)
bonesdeploy push

# Re-run remote setup to apply SSL configuration to Nginx
bonesdeploy remote setup
```

### SSL During Initial Setup
```bash
# During init, configure SSL in bones.yaml manually
bonesdeploy init

# Edit .bones/bones.yaml:
# ssl:
#   enabled: true
#   domain: app.example.com
#   email: admin@example.com

# Run remote setup (includes SSL)
bonesdeploy remote setup
bonesdeploy push
git push production master
```

---

## Let's Encrypt Rate Limits

Be aware of Let's Encrypt rate limits:
- **50 certificates per registered domain per week**
- **5 duplicate certificates per week**

**Testing:** Use Let's Encrypt's staging environment during testing:
```yaml
# In Ansible playbook
- name: Obtain staging certificate
  command: >
    certbot certonly --nginx
    --test-cert
    --non-interactive
    ...
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

4. **Rate limit exceeded**: Use staging environment or wait

### Renewal Issues

1. **Check renewal status**:
   ```bash
   sudo certbot renew --dry-run
   ```

2. **Check timer**:
   ```bash
   sudo systemctl status certbot.timer
   ```

---

## Related Commands

- `bonesdeploy remote setup` - Full server provisioning
- `bonesdeploy init` - Initialize project configuration
- `bonesdeploy push` - Sync configuration to remote
- `bonesdeploy doctor` - Validate environment

## See Also

- [Let's Encrypt Documentation](https://letsencrypt.org/docs/)
- [Certbot Documentation](https://certbot.eff.org/docs/)
