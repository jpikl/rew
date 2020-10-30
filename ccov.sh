#!/usr/bin/env bash

set -euo pipefail

main() {
  setup_environment "$@"
  build_and_test
  generate_coverage

  if [[ $MODE == ci ]]; then
    upload_coverage
  else
    report_coverage
  fi
}

setup_environment() {
  print_header "SETING UP ENVIRONMENT"

  readonly MODE=${1-dev}

  readonly GRCOV=$(detect_binary grcov)
  readonly RUST_COVFIX=$(detect_binary rust-covfix)

  readonly SOURCE_DIR=.
  readonly TARGET_DIR=./target/debug

  readonly COV_DIR=$TARGET_DIR/coverage
  readonly COV_INPUT=$COV_DIR/cov.zip
  readonly COV_OUTPUT=$COV_DIR/cov.info
  readonly COV_REPORT_DIR=$COV_DIR/report

  export CARGO_INCREMENTAL=0
  export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort"
  export RUSTDOCFLAGS="-Cpanic=abort"

  print_var MODE
  echo
  print_var GRCOV
  print_var RUST_COVFIX
  echo
  print_var SOURCE_DIR
  print_var TARGET_DIR
  echo
  print_var COV_DIR
  print_var COV_INPUT
  print_var COV_OUTPUT
  print_var COV_REPORT_DIR
  echo
  print_var CARGO_INCREMENTAL
  print_var RUSTFLAGS
  print_var RUSTDOCFLAGS

  print_header "TOOL VERSIONS"

  cargo +nightly --version
  $GRCOV --version
  printf "rust-covfix "
  $RUST_COVFIX --version
}

detect_binary() {
  if [[ -x ./$1 ]]; then
    echo "./$1"
  elif [[ -x "$(command -v "$1")" ]]; then
    echo "$1"
  else
    die "Unable to locate $1 binary"
  fi
}

build_and_test() {
  print_header "CARGO CLEAN"
  cargo +nightly clean

  print_header "CARGO BUILD"
  cargo +nightly build

  print_header "CARGO TEST"
  cargo +nightly test
}

generate_coverage() {
  print_header "PREPARING COVERAGE INPUT"

  rm -rf $COV_DIR
  mkdir -p $COV_DIR
  zip -0 $COV_INPUT $TARGET_DIR/deps/{rew,cpb,mvb}*.{gcda,gcno}

  print_header "GENERATING COVERAGE OUTPUT"

  $GRCOV $COV_INPUT \
  --source-dir $SOURCE_DIR \
  --llvm \
  --branch \
  --ignore-not-existing \
  --ignore "/*" \
  --ignore "tests/*" \
  --output-path $COV_OUTPUT

  print_header "FIXING COVERAGE OUTPUT"

  $RUST_COVFIX --verbose --output $COV_OUTPUT $COV_OUTPUT
}

upload_coverage() {
  print_header "UPLOADING COVERAGE"
  bash <(curl --silent https://codecov.io/bash) -f $COV_OUTPUT
}

report_coverage() {
  print_header "GENERATING COVERAGE REPORT"

  genhtml $COV_OUTPUT \
    --legend \
    --highlight \
    --show-details \
    --prefix "$(realpath src)" \
    --ignore-errors source \
    --output-directory $COV_REPORT_DIR

  open $COV_REPORT_DIR/index.html
}

open() {
   if [[ -x "$(command -v xdg-open)" ]]; then
     xdg-open "$1"
   elif [[ ${MSYSTEM-} =~ ^MINGW(32|64)$ && -x "$(command -v start)" ]]; then
     start "$1"
   else
     echo >&2 "Unable to detect command to open '$1'"
   fi
}

print_header() {
  echo
  echo "================================================================================"
  echo " $1"
  echo "================================================================================"
  echo
}

print_var() {
  echo "$1=${!1}"
}

die() {
  echo >&2 "$1"
  exit 1
}

main "$@"
