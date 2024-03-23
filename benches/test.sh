#!/usr/bin/env sh

set -eu

. ./benches/command.sh

# Tests output consistency of benchmarked commands.
#
# Usage: bench <cmd>...
#   <cmd>... Shell command(s) to be tested
bench() {
  NUMBER=$((NUMBER + 1))

  if [ "$NO_TEST" ]; then
    reset
    return
  fi

  echo
  printf "%bTesting %s benchmark #%s%b\n" "$BLUE" "$COMMAND" "$NUMBER" "$RESET"
  sh -c "$SETUP"

  FIRST="$1"
  FIRST_RESULT=$(sh -c "$FIRST | cksum")
  shift

  echo "$FIRST | cksum"
  echo "    $FIRST_RESULT"

  for OTHER; do
    OTHER_RESULT=$(sh -c "$OTHER | cksum")

    echo "$OTHER | cksum"
    echo "    $OTHER_RESULT"

    if [ "$FIRST_RESULT" != "$OTHER_RESULT" ]; then
      printf "\n%berror%b: %b%s%b produced a different output than %b%s%b\n" \
        "$RED" "$RESET" \
        "$BLUE" "$OTHER" "$RESET" \
        "$BLUE" "$FIRST" "$RESET"
      sh -c "$CLEANUP"
      exit 1
    fi
  done

  printf "%bSUCCESS%b\n" "$GREEN" "$RESET"
  sh -c "$CLEANUP"
  reset
}

# shellcheck disable=SC1090
. "./benches/commands/$COMMAND.sh"
