#!/bin/sh
set -ex

has_target() {
  rustup target list --installed | grep -q "$1"
}
ensure_target() {
  has_target "$1" || rustup target add "$1"
}
cargo_check() {
  cargo check "$@"
  # TODO: Uncomment once clippy lints are fixed.
  # cargo clippy "$@" -- --deny=warnings
}
cargo_test() {
  cargo_check --all-targets "$@"
  cargo test "$@"
}

cargo_test --features=alloc,experimental-derive

ensure_target thumbv7em-none-eabi
cargo_check --target=thumbv7em-none-eabi --no-default-features
cargo_check --target=thumbv7em-none-eabi --features=alloc,experimental-derive

cargo fmt -- --check

# TODO: Uncomment once documentation lints are fixed.
# env RUSTDOCFLAGS='--cfg=docsrs --deny=warnings' cargo doc --no-deps
