# shellcheck shell=sh

bench_last() {
  INPUT=input.txt
  LINES=$((SCALE * 2000 * ${2})) # 2000 from ~5000 lines

  setup "rew loop $((SCALE * ${2})) <$1 >$INPUT"
  cleanup "rm $INPUT"

  bench \
    "tail -n $LINES <$INPUT" \
    "coreutils tail -n $LINES <$INPUT" \
    "rew last $LINES <$INPUT"
}

bench_last "$SVEJK_FILE" 100
bench_last "$RUR_FILE" 1000
