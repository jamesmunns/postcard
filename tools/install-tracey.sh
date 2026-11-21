#!/bin/bash
# Install a pinned, checksum-verified prebuilt tracey into $1 (default
# $RUNNER_TEMP/bin) and add it to $GITHUB_PATH when running in GitHub Actions.
set -euo pipefail

version=v1.4.0
sha256=41d1360015f670b5d985b296aa7e727e3dcdb7ca04fc7834eb35d969a82ca9e5
target=x86_64-unknown-linux-gnu
archive="tracey-${target}.tar.xz"

dest="${1:-${RUNNER_TEMP:?}/bin}"
mkdir -p "$dest"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

curl --proto '=https' --tlsv1.2 -fsSL -o "$tmp/$archive" \
  "https://github.com/bearcove/tracey/releases/download/${version}/${archive}"
echo "${sha256}  $tmp/$archive" | sha256sum -c -
tar -xJf "$tmp/$archive" -C "$dest" --strip-components=1 "tracey-${target}/tracey"

if [ -n "${GITHUB_PATH:-}" ]; then
  echo "$dest" >> "$GITHUB_PATH"
fi
