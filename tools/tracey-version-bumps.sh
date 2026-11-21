#!/bin/bash
# Fail if any requirement text changed since base commit $1 without a
# version bump. Moves HEAD to $1 (keeping the index), so run it last.
set -euo pipefail

base="${1:-}"
if [ -z "$base" ] || [ "$base" = "0000000000000000000000000000000000000000" ]; then
  echo "No base commit to compare against"
  exit 0
fi

# `tracey pre-commit` checks staged changes, so stage everything since the
# base commit and let it compare against that.
git reset --soft "$base"
tracey pre-commit
