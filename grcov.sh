#!/usr/bin/env bash

set -euo pipefail

export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort"
export RUSTDOCFLAGS="-Cpanic=abort"

rm -rf ./target/debug/
cargo +nightly build
cargo +nightly test

grcov ./target/debug/ \
  --source-dir . \
  --output-type html \
  --llvm \
  --branch \
  --ignore-not-existing \
  --output-path ./target/debug/coverage/

open() {
   if [[ -x "$(command -v xdg-open)" ]]; then
     xdg-open "$1"
   elif [[ ${MSYSTEM-} =~ ^MINGW(32|64)$ && -x "$(command -v start)" ]]; then
     start "$1"
   else
     echo >&2 "Unable to detect command to open '$1'"
   fi
}

open ./target/debug/coverage/index.html