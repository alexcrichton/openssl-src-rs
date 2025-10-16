#!/bin/bash

target=$1
testcrate_dir="$(pwd)/testcrate"

set -ex

export CARGO_TARGET_AARCH64_APPLE_DARWIN_RUNNER=echo

cargo test --manifest-path "$testcrate_dir/Cargo.toml" --target $target -vvv
cargo test --manifest-path "$testcrate_dir/Cargo.toml" --target $target -vvv --release

if [ "$1" = "x86_64-unknown-linux-gnu" ]; then
    cargo test --manifest-path "$testcrate_dir/Cargo.toml" --target $target -vvv --all-features
fi
