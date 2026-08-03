#!/usr/bin/env bash
set -euo pipefail

: "${BONESDEPLOY_SITE:?missing BONESDEPLOY_SITE}"
: "${BONESDEPLOY_BONES_REPO:?missing BONESDEPLOY_BONES_REPO}"

repo="$BONESDEPLOY_BONES_REPO"
old_repo="/home/git/${BONESDEPLOY_SITE}.bones.git"
mkdir -p "$(dirname "$repo")"

if [ ! -e "$repo" ] && [ -e "$old_repo" ]; then
	mv "$old_repo" "$repo"
fi
if [ ! -d "$repo" ]; then
	git init --bare "$repo"
fi
chown -R root:root "$repo"
git --git-dir "$repo" symbolic-ref HEAD refs/heads/master

hook="$repo/hooks/pre-receive"
rm -f "$repo/hooks/post-receive"
cat >"$hook" <<'HOOK'
#!/usr/bin/env bash
set -euo pipefail

GIT_DIR="${GIT_DIR:-.}"
GIT_DIR=$(cd "$GIT_DIR" && pwd)
SITE=$(basename "$GIT_DIR")
SITE=${SITE%.bones.git}

while read -r _oldrev newrev refname; do
    if [ "$refname" = "refs/heads/master" ] && [ "$newrev" != "0000000000000000000000000000000000000000" ]; then
        exec bonesremote site receive --site "$SITE" --revision "$newrev"
    fi
done
HOOK
chown root:root "$hook"
chmod 0755 "$hook"
