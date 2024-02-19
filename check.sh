#!/usr/bin/env sh

set -eu

run() {
    echo -e "\033[33m> $@\033[0m"
    $@
    echo -e "\033[32m> SUCCESS\033[0m"
    echo
}

run cargo +nightly fmt --all

for arg in "" --all-features; do
    run cargo clippy --workspace $arg -- \
        -D clippy::all \
        -D clippy::pedantic \
        -A clippy::module_name_repetitions \
        -A clippy::must_use_candidate
done

run cargo test --quiet --all-features --all-targets
run cargo xtask docs
