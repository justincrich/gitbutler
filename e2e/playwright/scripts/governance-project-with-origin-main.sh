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
id = "dev"
permissions = ["administration:read", "contents:write"]

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

committed_permissions=$(git show HEAD:.gitbutler/permissions.toml)
dev_principal_block=$(
	awk '
		/^\[\[principal\]\]/ {
			if (capture) print block
			capture = 0
			block = ""
		}
		/^\[\[/ && $0 !~ /^\[\[principal\]\]/ {
			if (capture) print block
			capture = 0
			block = ""
		}
		{
			block = block $0 "\n"
			if ($0 == "id = \"dev\"") capture = 1
		}
		END {
			if (capture) print block
		}
	' <<<"$committed_permissions"
)

if [ -z "$dev_principal_block" ]; then
	echo "committed governance config is missing the dev principal" >&2
	exit 1
fi

if grep -Fq '"administration:write"' <<<"$dev_principal_block"; then
	echo "dev principal must not be committed with administration:write" >&2
	exit 1
fi

popd

git clone remote-project local-clone
pushd local-clone
git checkout main
"$BUT" setup
"$BUT" config target origin/main
popd
