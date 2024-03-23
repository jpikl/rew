# shellcheck shell=sh

bench_join() {
  INPUT=input.txt

  setup "rew loop $((SCALE * ${2})) <$1 >$INPUT"
  cleanup "rm $INPUT"

  bench \
    "(tr '\n' ' ' && echo) <$INPUT" \
    "rew join -t ' ' <$INPUT"
}

bench_join "$SVEJK_FILE" 50
