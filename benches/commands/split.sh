# shellcheck shell=sh

bench_split() {
  CHAR=$3
  INPUT=input.txt

  setup "rew loop $((SCALE * ${2})) <$1 >$INPUT"
  cleanup "rm $INPUT"

  bench \
    "tr '$CHAR' '\n' <$INPUT" \
    "rew split '$CHAR' <$INPUT"
}

bench_split "$SVEJK_FILE" 50 ','
bench_split "$SVEJK_FILE" 50 ' '
