```bash
#!/usr/bin/env bash

set -Eeuo pipefail

command -v python3 >/dev/null 2>&1 || { echo "python3 not found"; exit 1; }

# Activate virtualenv
VENV_DIR="${VENV_DIR:-venv}"
if [ -d "./$VENV_DIR" ]; then
  # shellcheck disable=SC1090
  source "./$VENV_DIR/bin/activate"
else
  echo "Virtual environment not found at ./$VENV_DIR"
  echo "Create one on the server: python3 -m venv $VENV_DIR"
  exit 1
fi

# Install dependencies
if [ -f "./requirements.txt" ]; then
  pip install -r requirements.txt --quiet
elif [ -f "./pyproject.toml" ]; then
  pip install . --quiet
fi

# Run migrations
python3 manage.py migrate --noinput

# Collect static files
python3 manage.py collectstatic --noinput

# Restart gunicorn via systemd
# Adjust the service name to match your systemd unit
SERVICE_NAME="${PROJECT_NAME:-gunicorn}"
if systemctl is-active --quiet "$SERVICE_NAME" 2>/dev/null; then
  sudo systemctl restart "$SERVICE_NAME"
elif systemctl list-unit-files | grep -q "$SERVICE_NAME"; then
  sudo systemctl start "$SERVICE_NAME"
else
  echo "No systemd service found for $SERVICE_NAME. Restart your app server manually."
fi
```
