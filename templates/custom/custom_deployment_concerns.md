```bash
#!/usr/bin/env bash

set -Eeuo pipefail

# Custom deployment script.
# Add your deployment commands below.
#
# This script runs on the remote server from the worktree directory
# after the latest code has been checked out.
#
# Available environment variables:
#   GIT_DIR       — path to the bare repo
#   PROJECT_NAME  — project name from bones.toml (if set)
#
# Examples:
#   npm ci && npm run build
#   bundle install && rails db:migrate
#   pip install -r requirements.txt && python manage.py migrate

echo "No deployment commands configured. Edit this script to add your own."
```
