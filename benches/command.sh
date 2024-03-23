# shellcheck shell=sh
# shellcheck disable=SC2034

# ANSI color codes
RED="\e[1;31m"
GREEN="\e[1;32m"
BLUE="\e[1;34m"
RESET="\e[0m"

COMMAND=$1 # Current command name
NUMBER=0   # Current benchmark number

# Disable output consistency test for the current benchmark.
#
# Usage: no_test <msg>
#   <msg> Message describing why the test is disabled.
no_test() {
  NO_TEST=$1
}

# Sets a command to be executed before benchmark.
#
# Usage: setup <cmd>
#   <cmd> Shell command to be executed.
setup() {
  SETUP=$1
}

# Sets a command to be executed after benchmark.
#
# Usage: setup <cmd>
#   <cmd> Shell command to be executed.
cleanup() {
  CLEANUP=$1
}

# Resets benchmark configuration.
reset() {
  NO_TEST=
  SETUP=
  CLEANUP=
}

reset
