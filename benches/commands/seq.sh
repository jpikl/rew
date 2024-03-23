# shellcheck shell=sh

bench_seq() {
  FROM=1
  TO=$((SCALE * ${1} * 1000000))
  STEP=$((SCALE * ${1}))

  bench \
    "seq $FROM $STEP $TO" \
    "coreutils seq $FROM $STEP $TO" \
    "rew seq $FROM..$TO $STEP"
}

bench_seq 1
bench_seq 10
bench_seq 100
