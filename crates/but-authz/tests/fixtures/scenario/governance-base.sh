#!/usr/bin/env bash

set -eu -o pipefail

git init -b main

echo "base" > README.md
git add README.md
git commit -m "base"
