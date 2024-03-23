# shellcheck shell=sh

bench_trim() {
  INPUT=input.txt

  setup "rew loop $((SCALE * ${2})) <$1 >$INPUT"
  cleanup "rm $INPUT"

  bench \
    "rew trim <$INPUT" \
    "awk '{ gsub(/^\s+|\s+$/, \"\"); print }' <$INPUT" \
    "sed -E 's/^\s+|\s+$//' <$INPUT"
}

bench_trim "$SVEJK_FILE" 10
bench_trim "$RUR_FILE" 100
