# ⌨️ Input

By default, input values are read as lines from standard input.
Each line is expected to be terminated either by `LF` or `CR+LF` characters.
The last line (before `EOF`) does not need to have a terminator.
 
- Use `-t, --read` option to read values terminated by a specific character.
- Use `-z, --read-nul` flag to read values terminated by `NUL` character.
- Use `-r, --read-raw` flag to read whole input into memory as a single value.
- Use `-l, --read-end` flag to read the last value (before `EOF`) only if it is properly terminated.

The following table shows how an input would be parsed for valid combinations of flags/options:

| Input    | *(no flag)* | `-l`     | `-z`     | `-lz`    | `-t:`    | `-lt:`   | `-r`    |
| -------- | ----------- | -------- | -------- | -------- | -------- | -------- | ------- |
| `a\nb`   | `a`, `b`    | `a`      | `a\nb`   | *(none)* | `a\nb`   | *(none)* |`a\nb`   | 
| `a\nb\n` | `a`, `b`    | `a`, `b` | `a\nb\n` | *(none)* | `a\nb\n` | *(none)* |`a\nb\n` |
| `a\0b`   | `a\0b`      | *(none)* | `a`, `b` | `a`      | `a\0b`   | *(none)* |`a\0b`   |
| `a\0b\0` | `a\0b\0`    | *(none)* | `a`, `b` | `a`, `b` | `a\0b\0` | *(none)* |`a\0b\0` |
| `a:b`    | `a:b`       | *(none)* | `a:b`    | *(none)* | `a`, `b` | `a`      |`a:b`    |
| `a:b:`   | `a:b:`      | *(none)* | `a:b:`   | *(none)* | `a`, `b` | `a`, `b` |`a:b:`   |

Input values can be also passed as additional arguments.
In such case, standard input will not be read.

```bash
rew '{}' image.jpg *.txt # Wildcard expansion is done by shell
```

Use flag `-I, --no-stdin` to enforce this behaviour even if there are no additional arguments.

```bash
echo a | rew '{}'       # Will print "a"
echo a | rew '{}' b     # Will print "b"
echo a | rew -I '{}'    # Will print nothing
echo a | rew -I '{}' b  # Will print "b"
```