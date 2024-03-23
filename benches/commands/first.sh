# shellcheck shell=sh

bench_first() {
  INPUT=input.txt
  LINES=$((SCALE * 2000 * ${2})) # 2000 from ~5000 lines

  setup "rew loop $((SCALE * ${2})) <$1 >$INPUT"
  cleanup "rm $INPUT"

  bench \
    "head -n $LINES <$INPUT" \
    "coreutils head -n $LINES <$INPUT" \
    "rew first $LINES <$INPUT"
}

bench_first "$SVEJK_FILE" 100
bench_first "$RUR_FILE" 1000
