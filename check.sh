#!/usr/bin/env sh

set -eu

cargo +nightly fmt
cargo check
cargo clippy
cargo test -q
