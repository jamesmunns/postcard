#!/bin/bash
# Install a pinned, checksum-verified prebuilt mdbook into $1 (default
# $RUNNER_TEMP/bin) and add it to $GITHUB_PATH when running in GitHub Actions.
set -euo pipefail

# mdbook 0.5 rejects `multilingual` in spec/mdbook/book.toml, so stay on 0.4.
version=v0.4.52
sha256=c0b903f01dd8f4edc644372ad2b80b1fdddd12552d37b6a098657cbd8eddd768
archive="mdbook-${version}-x86_64-unknown-linux-gnu.tar.gz"

dest="${1:-${RUNNER_TEMP:?}/bin}"
mkdir -p "$dest"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

curl --proto '=https' --tlsv1.2 -fsSL -o "$tmp/$archive" \
  "https://github.com/rust-lang/mdBook/releases/download/${version}/${archive}"
echo "${sha256}  $tmp/$archive" | sha256sum -c -
tar -xzf "$tmp/$archive" -C "$dest" mdbook

if [ -n "${GITHUB_PATH:-}" ]; then
  echo "$dest" >> "$GITHUB_PATH"
fi
