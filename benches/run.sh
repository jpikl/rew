#!/usr/bin/env sh

set -eu

# Integer size multiplier for all benchmark inputs (to make them run longer).
export SCALE=1

# Benchmark input filenames.
export SVEJK_FILE=svejk.txt
export RUR_FILE=rur.txt

usage() {
  echo "Usage:"
  echo "  $0               Run all targets"
  echo "  $0 summary       Update benchmark docs summary"
  echo "  $0 setup         Run benchmarks setup"
  echo "  $0 bench         Run benchmarks for all commands"
  echo "  $0 bench [name]  Run benchmarks for a specific command (cat, trim, ...)"
  echo "  $0 test          Check that all commands produce consistent output"
  echo "  $0 test [name]   Check that a specific command produce consistent output"
  echo "  $0 help          Print this help"
}

die() {
  echo "$0: $1"
  exit 1
} >&2

die_usage() {
  echo "$0: invalid usage"
  echo
  usage
  exit 1
} >&2

run() {
  if [ $# -gt 0 ]; then
    run_target "$@"
  else
    run_all
  fi
}

run_target() {
  TARGET=$1
  shift

  case "$TARGET" in
  setup) setup ;;
  summary) summary ;;
  bench) bench "$@" ;;
  test) test "$@" ;;
  -h | --help | help) usage ;;
  *) die_usage ;;
  esac
}

run_all() {
  test_all
  echo
  summary
  setup
  bench_all
}

summary() {
  FILE=./docs/SUMMARY.md
  echo "Updating summary in $FILE"
  ./benches/summary.sh <"$FILE" >"$FILE.new"
  mv "$FILE.new" "$FILE"
}

setup() {
  FILE=docs/benchmarks/setup.md
  echo "Generating setup to $FILE"
  ./benches/setup.sh >"$FILE"
}

bench() {
  if [ $# -gt 0 ]; then
    bench_one "$1"
  else
    bench_all
  fi
}

bench_one() {
  FILE="benches/commands/$1.sh"

  if [ -f "$FILE" ]; then
    ./benches/bench.sh "$1" >"docs/benchmarks/rew-$1.md"
  else
    die "'$FILE' does not exist"
  fi
}

bench_all() {
  for FILE in ./benches/commands/*.sh; do
    bench_one "$(basename "$FILE" .sh)"
  done
}

test() {
  if [ $# -gt 0 ]; then
    test_one "$1"
  else
    test_all
  fi
}

test_one() {
  ./benches/test.sh "$1"
}

test_all() {
  for FILE in ./benches/commands/*.sh; do
    test_one "$(basename "$FILE" .sh)"
  done
}

run "$@"
