# 💬 Output

By default, results are printed as lines to standard output.
`LF` character is used as a line terminator.

- Use `-T, --print` option to print results terminated by a specific string.
- Use `-Z, --print-nul` flag to print results terminated by NUL character.
- Use `-R, --print-raw` flag to print results without a terminator.
- Use `-L, --no-print-last` flag to disable printing terminator for the last result.

```bash
rew '{D}' | xargs mkdir -p       # Pass extracted directories to mkdir command
rew -Z '{D}' | xargs -0 mkdir -p # Use NUL terminator in case paths contain newlines
rew -T$'\r\n'                    # Convert newlines to CR+LF using custom output terminator
rew -R '{}%r%n'                  # Same thing but output terminator is inside pattern
rew -LT+ '{}' a b c              # Join input values to string "a+b+c"
```

Apart from this (standard) mode, there are also two other output modes.

## 🤖 Diff mode

- Enabled using `-b, --diff` flag.
- Respects `--print*` flags/options.l
- Ignores `--no-print-last` flag.
- Prints machine-readable transformations as results:

```text
<input_value_1
>output_value_1
<input_value_2
>output_value_2
...
<input_value_N
>output_value_N
```

Such output can be processed by accompanying `mvb` and `cpb` utilities to perform bulk move/copy.

```bash
find -name '*.jpeg' | rew -b '{B}.jpg' | mvb # Rename all *.jpeg files to *.jpg
find -name '*.txt'  | rew -b '{}.bak'  | cpb # Make backup copy of each *.txt file
```

## 🌹 Pretty mode

- Enabled using `-p, --pretty` flag.
- Ignores `--print*` flags/options.
- Ignores `--no-print-last` flag.
- Prints human-readable transformations as results:

```text
input_value_1 -> output_value_1
input_value_2 -> output_value_2
...
input_value_N -> output_value_N
```
