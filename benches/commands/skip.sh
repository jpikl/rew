# shellcheck shell=sh

bench_skip() {
  INPUT=input.txt
  LINES=$((SCALE * 2000 * ${2})) # 2000 from ~5000 lines

  setup "rew loop $((SCALE * ${2})) <$1 >$INPUT"
  cleanup "rm $INPUT"

  bench \
    "tail -n +$((LINES + 1)) <$INPUT" \
    "coreutils tail -n +$((LINES + 1)) <$INPUT" \
    "rew skip $LINES <$INPUT"
}

bench_skip "$SVEJK_FILE" 100
bench_skip "$RUR_FILE" 1000
