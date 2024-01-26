#!/usr/bin/env sh

set -eu

cargo +nightly fmt
cargo clippy -- -D clippy::all -D clippy::pedantic -A clippy::module_name_repetitions
cargo test -q
