```bash
#!/usr/bin/env bash

set -Eeuo pipefail

command -v ruby >/dev/null 2>&1 || { echo "ruby not found"; exit 1; }
command -v bundle >/dev/null 2>&1 || { echo "bundler not found"; exit 1; }

# Load rbenv if available
if [ -d "$HOME/.rbenv" ]; then
  export PATH="$HOME/.rbenv/bin:$PATH"
  eval "$(rbenv init -)"
fi

# Install Ruby version from .ruby-version if rbenv is available
if [ -f "./.ruby-version" ] && command -v rbenv >/dev/null 2>&1; then
  rbenv install --skip-existing
fi

# Install dependencies
bundle install --deployment --without development test

# Precompile assets
if bundle exec rails assets:precompile 2>/dev/null; then
  echo "Assets precompiled."
fi

# Run database migrations
bundle exec rails db:migrate RAILS_ENV=production

# Restart puma via systemd
# Adjust the service name to match your systemd unit
SERVICE_NAME="${PROJECT_NAME:-puma}"
if systemctl is-active --quiet "$SERVICE_NAME" 2>/dev/null; then
  sudo systemctl restart "$SERVICE_NAME"
elif systemctl list-unit-files | grep -q "$SERVICE_NAME"; then
  sudo systemctl start "$SERVICE_NAME"
else
  echo "No systemd service found for $SERVICE_NAME. Restart your app server manually."
fi
```
