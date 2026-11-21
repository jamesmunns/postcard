#!/bin/bash
# Install a pinned, checksum-verified prebuilt tracey into $1 (default
# $RUNNER_TEMP/bin) and add it to $GITHUB_PATH when running in GitHub Actions.
set -euo pipefail

version=v1.4.0
# Pick the prebuilt for this machine.
#
# GHA uses x86_64, the deploy server uses aarch64.
case "$(uname -m)" in
  x86_64)
    target=x86_64-unknown-linux-gnu
    sha256=41d1360015f670b5d985b296aa7e727e3dcdb7ca04fc7834eb35d969a82ca9e5
    ;;
  aarch64)
    target=aarch64-unknown-linux-gnu
    sha256=c0cebdcd76c1255b0251075c6828755bb142e41012d0d7d868b32237a027a8dd
    ;;
  *)
    echo "install-tracey: unsupported architecture $(uname -m)" >&2
    exit 1
    ;;
esac
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
