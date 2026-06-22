#!/bin/bash

set -euo pipefail

echo "GIT CONFIG $GIT_CONFIG_GLOBAL"
echo "DATA DIR $E2E_TEST_APP_DATA_DIR"
echo "BUT $BUT"

mkdir remote-project
pushd remote-project
git init -b main --object-format=sha1
echo "governance e2e" > README.md
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "admin"
permissions = ["administration:read", "administration:write", "merge"]

[[principal]]
id = "admin-readonly"
permissions = ["administration:read"]

[[principal]]
id = "test-principal"
permissions = ["contents:read"]

[[group]]
name = "test-group"
permissions = ["contents:write"]
members = ["test-principal", "group-principal"]
EOF
cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true
EOF
git add README.md
git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "Initial governance E2E project"
popd

git clone remote-project local-clone
pushd local-clone
git checkout main
"$BUT" setup
"$BUT" config target origin/main
popd
