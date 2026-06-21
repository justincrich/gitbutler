#!/bin/bash

set -euo pipefail

echo "GIT CONFIG $GIT_CONFIG_GLOBAL"
echo "DATA DIR $E2E_TEST_APP_DATA_DIR"
echo "BUT $BUT"

mkdir remote-project
pushd remote-project
git init -b main --object-format=sha1
echo "governance e2e" > README.md
git add README.md
git commit -m "Initial governance E2E project"
popd

git clone remote-project local-clone
pushd local-clone
git checkout main
"$BUT" setup
"$BUT" config target origin/main
popd
