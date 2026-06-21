#!/usr/bin/env bash

set -euo pipefail

if [ "$#" -ne 1 ]; then
	echo "usage: $0 <fresh-workdir>" >&2
	exit 64
fi

workdir=$1

mkdir -p "$workdir"
git init -b master --object-format=sha1 "$workdir" >/dev/null
git -C "$workdir" config user.name "GitButler Governance E2E"
git -C "$workdir" config user.email "governance-e2e@example.com"
git -C "$workdir" config commit.gpgsign false

mkdir -p "$workdir/.gitbutler"
cat >"$workdir/.gitbutler/permissions.toml" <<'EOF'
[[principal]]
id = "admin"
permissions = ["administration:write", "merge"]
groups = ["maintainers"]

[[principal]]
id = "dev"
permissions = ["contents:write"]
groups = ["code-reviewers"]

[[group]]
name = "maintainers"
permissions = []
members = ["admin"]

[[group]]
name = "code-reviewers"
permissions = []
members = ["dev"]
EOF

cat >"$workdir/.gitbutler/gates.toml" <<'EOF'
[[branch]]
name = "master"
protected = true
EOF

git -C "$workdir" add .gitbutler/permissions.toml .gitbutler/gates.toml
git -C "$workdir" commit -m "Seed governance E2E config" >/dev/null

if [ -n "$(git -C "$workdir" status --porcelain)" ]; then
	echo "seed-governance.sh left the worktree dirty" >&2
	git -C "$workdir" status --porcelain >&2
	exit 1
fi
