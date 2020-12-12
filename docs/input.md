# ⌨️ Input

By default, input values are read as lines from standard input.
`LF` or `CR+LF` is auto-detected as a delimiter, independent of platform.

- Use `-z, --read-nul` flag to read values delimited by NUL character.
- Use `-r, --read-raw` flag to read whole input into memory as a single value.
- Use `-d, --read` option to read values delimited by a specific character.

```bash
find | rew '{a}'            # Convert output of find command to absolute paths
find -print0 | rew -z '{a}' # Use NUL delimiter in case paths contain newlines
echo "$PATH" | rew -d:      # Split PATH variable entries delimited by colon
rew -r 'A{}B' <data.txt     # Read file as a whole, prepend 'A', append 'B'
```

Input values can be also passed as additional arguments, after a pattern.

```bash
rew '{a}' *.txt # Wildcard expansion is done by shell
```
