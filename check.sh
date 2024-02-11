#!/usr/bin/env sh

set -eu

cargo +nightly fmt

cargo clippy -- \
    -D clippy::all \
    -D clippy::pedantic \
    -A clippy::module_name_repetitions \
    -A clippy::must_use_candidate \
    -A clippy::missing_panics_doc \
    -A clippy::missing_errors_doc

cargo test -q
