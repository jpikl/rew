# shellcheck shell=sh

bench_stream() {
  INPUT=input.txt
  FROM=1
  TO=$((SCALE * ${1}))

  setup "seq $FROM $TO >$INPUT"
  cleanup "rm $INPUT"

  bench \
    "rew stream {$FROM..$TO}" \
    "printf '%s\n' {$FROM..$TO}" \
    "xargs rew stream <$INPUT" \
    "xargs printf '%s\n' <$INPUT"
}

bench_stream 100000
