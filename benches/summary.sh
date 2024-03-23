#!/usr/bin/env sh

set -eu

while read -r LINE; do
  echo "$LINE"
  if [ "$LINE" = "<!--[GENERATED_BENCHMARKS_START]-->" ]; then
    echo
    break
  fi
done

echo "- [Setup](./benchmarks/setup.md)"

for FILE in ./benches/commands/*.sh; do
  COMMAND=$(basename "$FILE" .sh)
  echo "- [rew $COMMAND](./benchmarks/rew-$COMMAND.md)"
done

while read -r LINE; do
  if [ "$LINE" = "<!--[GENERATED_BENCHMARKS_END]-->" ]; then
    echo
    echo "$LINE"
    break
  fi
done

cat
