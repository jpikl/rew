# shellcheck shell=sh

bench_cat() {
  INPUT=input.txt

  setup "rew loop $((SCALE * ${2})) <$1 >$INPUT"
  cleanup "rm $INPUT"

  bench \
    "cat <$INPUT" \
    "coreutils cat <$INPUT" \
    "rew cat <$INPUT" \
    "rew cat --bytes <$INPUT" \
    "rew cat --chars <$INPUT" \
    "rew cat --lines <$INPUT"
}

bench_cat "$SVEJK_FILE" 500
bench_cat "$RUR_FILE" 5000
