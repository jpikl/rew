# shellcheck shell=sh

bench_lower() {
  INPUT=input.txt

  setup "rew loop $((SCALE * ${2})) <$1 >$INPUT"
  cleanup "rm $INPUT"

  bench \
    "rew lower <$INPUT" \
    "sed 's/\(.*\)/\L\1/' <$INPUT" \
    "while IFS= read -r l;do printf '%s\\n' \"\${l,,}\";done <$INPUT"
}

bench_lower_ascii() {
  INPUT=input.txt

  setup "rew ascii <$1 | rew loop $((SCALE * ${2})) >$INPUT"
  cleanup "rm $INPUT"

  bench \
    "rew lower <$INPUT" \
    "dd conv=lcase status=none <$INPUT" \
    "tr '[:upper:]' '[:lower:]' <$INPUT" \
    "sed 's/\(.*\)/\L\1/' <$INPUT"
}

bench_lower "$SVEJK_FILE" 1
bench_lower "$RUR_FILE" 10

bench_lower_ascii "$SVEJK_FILE" 20
bench_lower_ascii "$RUR_FILE" 100
