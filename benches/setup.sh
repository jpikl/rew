#!/usr/bin/env sh

set -eu

echo "# Benchmarks"
echo
echo "This section contains \`rew\` benchmarks against various other tools, like:"
echo
echo "- [uutils coreutils](https://github.com/uutils/coreutils)"
echo "- [GNU coreutils](https://www.gnu.org/software/coreutils/)"
echo "- [GNU bash](https://www.gnu.org/software/bash/)"
echo "- [GNU sed](https://www.gnu.org/software/sed/)"
echo "- [GNU awk](https://www.gnu.org/software/gawk/)"
echo
echo "## Environment"
echo
echo "All benchmarks were run using the following environment."
echo
echo "### System"
echo

specs() {
  echo "<details>"
  echo "<summary><code>$1</code></summary>"
  echo
  echo '```'
  LC_ALL=C sh -c "$1"
  echo '```'
  echo
  echo "</details>"
}

specs 'neofetch --stdout os distro kernel shell model cpu memory'
specs 'uname -a'
specs 'lscpu'
specs 'lsmem'
specs 'free -h'

echo
echo "### Software"
echo
echo '```'

hyperfine --version | head -n1
rew --version | head -n1
coreutils --help | head -n1
awk --version | head -n1
bash --version | head -n1
dd --version | head -n1
head --version | head -n1
sed --version | head -n1
seq --version | head -n1
tail --version | head -n1
tr --version | head -n1

echo '```'

BASE_URL=https://jpikl.github.io/data

SVEJK_INIT="curl $BASE_URL/$SVEJK_FILE -o $SVEJK_FILE"
RUR_INIT="curl $BASE_URL/$RUR_FILE -o $RUR_FILE"

echo
echo "## Data"
echo
echo "We are using the following files as input for benchmarking:"
echo
echo "| | [$SVEJK_FILE]($BASE_URL/$SVEJK_FILE)  | [$RUR_FILE]($BASE_URL/$RUR_FILE) |"
echo "| ----------- | ------------------------- | ------------------- |"
echo "| Encoding    | UTF-8                     | UTF-8               |"
echo "| Characters  | Czech / German diacritics | Czech diacritics    |"
echo "| Size        | 1.31 MiB                  | 143 KiB             |"
echo "| Line count  | 5913                      | 4477                |"
echo "| Line width  | up to 4925 characters     | up to 81 characters |"
echo "| Whitespaces | Trimmed                   | Around lines        |"
echo
echo "Both files can be downloaded using the following commands:"
echo
echo '```shell'
echo "$SVEJK_INIT"
echo "$RUR_INIT"
echo '```'

[ -f "$SVEJK_FILE" ] || sh -c "$SVEJK_INIT"
[ -f "$RUR_FILE" ] || sh -c "$RUR_INIT"
