#!/usr/bin/env sh

set -eu

cargo +nightly fmt

for arg in "" --all-features; do
    cargo clippy $arg -- \
        -D clippy::all \
        -D clippy::pedantic \
        -A clippy::module_name_repetitions \
        -A clippy::must_use_candidate 
done

cargo test -q --all-features
