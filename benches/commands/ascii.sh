# shellcheck shell=sh

bench_ascii() {
  INPUT=input.txt

  setup "rew loop $((SCALE * ${2})) <$1 >$INPUT"
  cleanup "rm $INPUT"

  bench \
    "rew ascii <$INPUT"
}

bench_ascii_delete() {
  INPUT=input.txt

  setup "rew loop $((SCALE * ${2})) <$1 >$INPUT"
  cleanup "rm $INPUT"

  bench \
    "rew ascii -d <$INPUT" \
    "LC_ALL=C sed 's/[^\x00-\x7F]//g' <$INPUT"
}

bench_ascii "$SVEJK_FILE" 2
bench_ascii_delete "$SVEJK_FILE" 2
