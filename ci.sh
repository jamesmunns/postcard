#!/bin/sh
set -ex

has_target() {
  rustup target list --installed | grep -q "$1"
}
ensure_target() {
  has_target "$1" || rustup target add "$1"
}

ensure_target thumbv7em-none-eabi
ensure_target riscv32i-unknown-none-elf

has_toolchain() {
  rustup toolchain list | grep -q "$1"
}
ensure_toolchain() {
  has_toolchain "$1" || rustup toolchain install "$1"
}

ensure_toolchain nightly

cargo_check() {
  cargo check --all "$@"
  cargo clippy --all "$@" -- --deny=warnings
}
cargo_test() {
  cargo_check --all-targets "$@"
  cargo test --all "$@"
}

cargo_test --features=alloc,experimental-derive,use-std,use-crc

cargo_check --target=thumbv7em-none-eabi --no-default-features
cargo_check --target=thumbv7em-none-eabi --features=alloc,experimental-derive

# CC https://github.com/jamesmunns/postcard/issues/167 - don't accidentally use atomics
# on non-atomic systems
cargo_check --target=riscv32i-unknown-none-elf --features=alloc,experimental-derive

cargo fmt --all -- --check

# Check docs.rs build
env RUSTDOCFLAGS='--cfg=docsrs --deny=warnings' cargo +nightly doc --all --no-deps --all-features
