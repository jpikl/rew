# 💬 Output

By default, results are printed as lines to standard output.
`LF` is used as a delimiter.

- Use `-Z, --print-nul` flag to print results delimited by NUL character.
- Use `-R, --print-raw` flag to print results without a delimiter.
- Use `-D, --print` option to print results delimited by a specific string.
- Use `-T, --no-trailing-delimiter` flag to not print final delimiter at the end of output.

```bash
rew '{D}' | xargs mkdir -p       # Pass extracted directories to mkdir command
rew -Z '{D}' | xargs -0 mkdir -p # Use NUL delimiter in case paths contain newlines
rew -D$'\r\n'                    # Convert newlines to CR+LF using custom output delimiter
rew -R '{}#r#n'                  # Same thing but output delimiter is in the pattern
rew -TD+ '{}' a b c              # Join input values to string "a+b+c"
```

Apart from this (standard) mode, there are also two other output modes.

## 🤖 Diff mode

- Enabled using `-b, --diff` flag.
- Respects `--print*` flags/options.
- Ignores `--no-trailing-delimiter` flag.
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
- Ignores `--no-trailing-delimiter` flag.
- Prints human-readable transformations as results:

```text
input_value_1 -> output_value_1
input_value_2 -> output_value_2
...
input_value_N -> output_value_N
```
