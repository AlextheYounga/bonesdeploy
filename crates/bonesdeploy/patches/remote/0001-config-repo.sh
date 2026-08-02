#!/usr/bin/env bash
set -euo pipefail

: "${BONESDEPLOY_SITE:?missing BONESDEPLOY_SITE}"
: "${BONESDEPLOY_BONES_REPO:?missing BONESDEPLOY_BONES_REPO}"

repo="$BONESDEPLOY_BONES_REPO"
mkdir -p "$(dirname "$repo")"
if [ ! -d "$repo" ]; then
	git init --bare "$repo"
fi
chown -R git:git "$repo"
git --git-dir "$repo" symbolic-ref HEAD refs/heads/master

hook="$repo/hooks/post-receive"
cat >"$hook" <<'HOOK'
#!/usr/bin/env bash
set -euo pipefail

GIT_DIR="${GIT_DIR:-.}"
GIT_DIR=$(cd "$GIT_DIR" && pwd)
SITE=$(basename "$GIT_DIR")
SITE=${SITE%.bones.git}

while read -r _oldrev newrev refname; do
    if [ "$refname" = "refs/heads/master" ] && [ "$newrev" != "0000000000000000000000000000000000000000" ]; then
        exec sudo bonesremote site receive --site "$SITE" --revision "$newrev"
    fi
done
HOOK
chown git:git "$hook"
chmod 0755 "$hook"

sudoers=/etc/sudoers.d/bonesdeploy
temporary=$(mktemp)
if [ -f "$sudoers" ]; then
	sed '/^git ALL=(root) NOPASSWD:/d' "$sudoers" >"$temporary"
fi
cat >>"$temporary" <<'SUDOERS'
git ALL=(root) NOPASSWD: /usr/local/bin/bonesremote ^hook post-receive --site [a-z0-9-]+$, /usr/local/bin/bonesremote ^site receive --site [a-z0-9-]+ --revision [0-9a-f]{40}$, /usr/local/bin/bonesremote ^service restart --site [a-z0-9-]+$, /usr/local/bin/bonesremote ^release rollback --site [a-z0-9-]+$, /usr/local/bin/bonesremote ^release drop-failed --site [a-z0-9-]+$, /usr/local/bin/bonesremote ^release prune --site [a-z0-9-]+$
SUDOERS
chmod 0440 "$temporary"
chown root:root "$temporary"
visudo -cf "$temporary"
mv "$temporary" "$sudoers"
