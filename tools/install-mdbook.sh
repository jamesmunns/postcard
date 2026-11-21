#!/bin/bash
# Install a pinned, checksum-verified prebuilt mdbook into $1 (default
# $RUNNER_TEMP/bin) and add it to $GITHUB_PATH when running in GitHub Actions.
set -euo pipefail

# mdbook 0.5 rejects `multilingual` in spec/mdbook/book.toml, so stay on 0.4.
version=v0.4.52
# Pick the prebuilt for this machine; upstream has no aarch64 gnu build.
#
# GHA uses x86_64, the deploy server uses aarch64.
case "$(uname -m)" in
  x86_64)
    target=x86_64-unknown-linux-gnu
    sha256=c0b903f01dd8f4edc644372ad2b80b1fdddd12552d37b6a098657cbd8eddd768
    ;;
  aarch64)
    target=aarch64-unknown-linux-musl
    sha256=7273dda980915a1e2f114d63d432aa6284551e37f0358e3ce7653d1e49e6fa3f
    ;;
  *)
    echo "install-mdbook: unsupported architecture $(uname -m)" >&2
    exit 1
    ;;
esac
archive="mdbook-${version}-${target}.tar.gz"

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
