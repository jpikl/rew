#!/usr/bin/env sh

set -eu

cargo +nightly fmt
cargo clippy -- -D clippy::all -D clippy::pedantic
cargo test -q
