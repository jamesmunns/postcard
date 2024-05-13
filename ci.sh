#!/bin/sh
set -ex

has_target() {
  rustup target list --installed | grep -q "$1"
}
ensure_target() {
  has_target "$1" || rustup target add "$1"
}

ensure_target thumbv7em-none-eabi

has_toolchain() {
  rustup toolchain list | grep -q "$1"
}
ensure_toolchain() {
  has_toolchain "$1" || rustup toolchain install "$1"
}

ensure_toolchain nightly-x86_64-unknown-linux-gnu

cargo_check() {
  cargo check "$@"
  cargo clippy "$@" -- --deny=warnings
}
cargo_test() {
  cargo_check --all-targets "$@"
  cargo test "$@"
}

cargo_test --features=alloc,experimental-derive,use-std,use-crc

cargo_check --target=thumbv7em-none-eabi --no-default-features
cargo_check --target=thumbv7em-none-eabi --features=alloc,experimental-derive

cargo fmt -- --check

env RUSTDOCFLAGS='--cfg=docsrs --deny=warnings' cargo +nightly doc --no-deps --all-features

cd postcard-derive

cargo_check
cargo fmt -- --check
