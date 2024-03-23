# shellcheck shell=sh

bench_upper() {
  INPUT=input.txt

  # GNU coreutils do not know how to convert 'ß'
  setup "sed s/ß/s/g <$1 | rew loop $((SCALE * ${2})) >$INPUT"
  cleanup "rm $INPUT"

  bench \
    "rew upper <$INPUT" \
    "sed 's/\(.*\)/\U\1/' <$INPUT" \
    "while IFS= read -r l;do printf '%s\\n' \"\${l^^}\";done <$INPUT"
}

bench_upper_ascii() {
  INPUT=input.txt

  setup "rew ascii <$1 | rew loop $((SCALE * ${2})) >$INPUT"
  cleanup "rm $INPUT"

  bench \
    "rew upper <$INPUT" \
    "dd conv=ucase status=none <$INPUT" \
    "coreutils dd conv=ucase status=none <$INPUT" \
    "tr '[:lower:]' '[:upper:]' <$INPUT" \
    "coreutils tr '[:lower:]' '[:upper:]' <$INPUT" \
    "sed 's/\(.*\)/\U\1/' <$INPUT"
}

bench_upper "$SVEJK_FILE" 1
bench_upper "$RUR_FILE" 10

bench_upper_ascii "$SVEJK_FILE" 20
bench_upper_ascii "$RUR_FILE" 100
