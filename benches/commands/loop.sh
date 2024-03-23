# shellcheck shell=sh

bench_loop() {
  INPUT=input.txt
  LOOPS=10

  setup "rew loop $((SCALE * ${2})) <$1 >$INPUT"
  cleanup "rm $INPUT"

  bench \
    "rew loop $LOOPS <$INPUT" \
    "for((i=0;i<$LOOPS;i++)); do cat $INPUT; done"
}

bench_loop "$SVEJK_FILE" 10
