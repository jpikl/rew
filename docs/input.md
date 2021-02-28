# ⌨️ Input

By default, input values are read as lines from standard input.
Each line is expected to be terminated either by `LF` or `CR+LF` characters.
The last line (before `EOF`) does not need to have a terminator.

- Use `td, --read` option to read values terminated by a specific character.
- Use `-z, --read-nul` flag to read values terminated by NUL character.
- Use `-r, --read-raw` flag to read whole input into memory as a single value.

```bash
find | rew '{a}'            # Convert output of find command to absolute paths
find -print0 | rew -z '{a}' # Use NUL terminator in case paths contain newlines
echo "$PATH" | rew -t:      # Split PATH variable entries separated by colon
rew -r 'A{}B' <data.txt     # Read file as a whole, prepend 'A', append 'B'
```

Input values can be also passed as additional arguments.
In such case, standard input will not be read.

```bash
rew '{a}' *.txt # Wildcard expansion is done by shell
```

Use flag `-I, --no-stdin` to enforce this behaviour even if there are no additional arguments.

```bash
echo 'a' | rew '{}'     # Will print "a"
echo 'a' | rew '{}' 'b' # Will print "b"
echo 'a' | rew -I '{}'  # Will print nothing
```