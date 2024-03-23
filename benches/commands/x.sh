# shellcheck shell=sh

bench_x_pipe() {
  INPUT=input.txt
  CMD="cat"

  setup "rew loop $((SCALE * ${2})) <$1 >$INPUT"
  cleanup "rm $INPUT"

  bench \
    "rew x '{}' <$INPUT" \
    "rew x '{$CMD}' <$INPUT" \
    "rew x '{$CMD | $CMD}' <$INPUT" \
    "rew x '{$CMD | $CMD | $CMD}' <$INPUT" \
    "rew x '{$CMD | $CMD | $CMD | $CMD}' <$INPUT" \
    "rew x '{$CMD | $CMD | $CMD | $CMD | $CMD}' <$INPUT"
}

bench_x_expr() {
  INPUT=input.txt
  CMD="cat"

  no_test "Each benchmarked command produces different output."
  setup "rew loop $((SCALE * ${2})) <$1 >$INPUT"
  cleanup "rm $INPUT"

  bench \
    "rew x '{}' <$INPUT" \
    "rew x '{$CMD}' <$INPUT" \
    "rew x '{$CMD}{$CMD}' <$INPUT" \
    "rew x '{$CMD}{$CMD}{$CMD}' <$INPUT" \
    "rew x '{$CMD}{$CMD}{$CMD}{$CMD}' <$INPUT" \
    "rew x '{$CMD}{$CMD}{$CMD}{$CMD}{$CMD}' <$INPUT"
}

bench_x_pipe "$SVEJK_FILE" 100
bench_x_pipe "$RUR_FILE" 1000

bench_x_expr "$SVEJK_FILE" 100
bench_x_expr "$RUR_FILE" 1000
