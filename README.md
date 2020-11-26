# rew

Rew is a CLI tool that rewrites FS paths according to a pattern.

[![Build status][build-img]][build]
[![Code coverage][coverage-img]][coverage]

## Contents

- [:bulb: What rew does](#bulb-what-rew-does)
- [:package: Installation](#package-installation)
- [:keyboard: Input](#keyboard-input)
- [:pencil: Pattern](#pencil-pattern)
  - [:railway_track: Path filters](#railway_track-path-filters)
  - [:ab: Substring filters](#ab-substring-filters)
  - [:mag: Replace filters](#mag-replace-filters)
  - [:asterisk: Regex filters](#asterisk-regex-filters)
  - [:art: Format filters](#art-format-filters)
  - [:infinity: Generators](#infinity-generators)
- [:speech_balloon: Output](#speech_balloon-output)
  - [:robot: Diff mode](#robot-diff-mode)
  - [:rose: Pretty mode](#rose-pretty-mode)
- [:microscope: Comparison with similar tools](#microscope-comparison-with-similar-tools)
- [:rocket: Examples](#rocket-examples)
- [:page_facing_up: License](#page_facing_up-license)

## :bulb: What rew does

1. Reads values from standard input.
2. Rewrites them according to a pattern.
3. Prints results to standard output.

Input values are assumed to be FS paths, however, `rew` is able to process any UTF-8 encoded text.

![What rew does](images/diagram.png)

## :package: Installation

- Set up a [Rust development environment][rust-dev].
- Install `rew` from sources using `cargo`.

    ```bash
    cargo install --git https://github.com/jpikl/rew
    ```
- Binaries will be installed to `.cargo/bin/` in your home directory.

## :keyboard: Input

By default, values are read as lines from standard input.
`LF` or `CR+LF` is auto-detected as a delimiter, independent of platform.

- Use `-z, --read-nul` flag to read values delimited by NUL character.
- Use `-r, --read-raw` flag to read whole input into memory as a single value.
- Use `-d, --read` option to read values delimited by a specific character.

```bash
find         | rew    '{a}' # Convert output lines from find command to absolute paths
find -print0 | rew -z '{a}' # Use NUL delimiter in case paths contain newlines
echo "$PATH" | rew -d:      # Split PATH variable entries delimited by colon
rew -r 'A{}B' <data.txt     # Read file as a whole, prepend 'A', append 'B'
```

Input values can be also provided as additional arguments, after a pattern.

```bash
rew '{a}' *.txt # Wildcard expansion is done by shell
```

## :pencil: Pattern

Pattern is a string describing how to generate output from an input.

Use `--explain` flag to print detailed explanation what a certain pattern does.

```bash
rew --explain 'file_{c|<3:0}.{e}'
```

By default, pattern characters are directly copied to output.

| Input | Pattern | Output |
| ----- | ------- | ------ |
| *     | `abc`   | `abc`  |

Characters `{` and `}` form an expression which is evaluated and replaced in output.

Empty exrpession `{}` evaluates directly to input value.

| Input   | Pattern      | Output          |
| ------- | ------------ | --------------- |
| `world` | `{}`         | `world`         |
| `world` | `Hello_{}_!` | `Hello_world_!` |

Expression may contain one or more filters, delimited by `|`, which are consecutively applied on input value.

| Input      | Pattern           | Output     | Description                        |
| ---------- | ----------------- | ---------- | ---------------------------------- |
| `old.JPEG` | `new.{e}`         | `new.JPEG` | Extension                          |
| `old.JPEG` | `new.{e\|l}`      | `new.jpeg` | Extension + Lowercase              |
| `old.JPEG` | `new.{e\|l\|r:e}` | `new.jpg`  | Extension + Lowercase + Remove `e` |

Character `#` starts an escape sequence.

| Sequence | Description     |
| -------- |---------------  |
| `#n`     | New line        |
| `#r`     | Carriage return |
| `#t`     | Horizontal tab  |
| `#0`     | Null            |
| `#{`     | Escaped `{`     |
| `#\|`    | Escaped `\|`    |
| `#}`     | Escaped `{`     |
| `##`     | Escaped `#`     |

Use `--escape` option to set a different escape character.

```bash
rew '{R:#t: }'              # Replace tabs with spaces
rew '{R:\t: }' --escape='\' # Same thing, different escape character
```

If no pattern is provided, input values are directly copied to output.

```bash
printf 'a\0b' | rew -z # Convert NUL bytes to newlines
```

### :railway_track: Path filters

| Filter | Description             |
| ------ | ----------------------- |
| `a`    | Absolute path           |
| `p`    | Normalized path         |
| `P`    | Canonical path          |
| `d`    | Parent directory        |
| `D`    | Path without last name  |
| `f`    | File name               |
| `F`    | Last name               |
| `b`    | Base name               |
| `B`    | Path without extension  |
| `e`    | Extension               |
| `E`    | Extension with dot<br/>Dot is not printed for missing extension. |

Let us assume the following directory structure:

```text
/
└── home
    ├── alice
    │   └── notes.txt
    |
    └── bob
```

For working directory `/home/bob` and input `../alice/notes.txt`, filters would evaluate to:

| Filter | Output                         |
| ------ | ------------------------------ |
| `a`    | `/home/bob/../alice/notes.txt` |
| `p`    | `../alice/notes.txt`           |
| `P`    | `/home/alice/notes.txt`        |
| `d`    | `../alice`                     |
| `D`    | `../alice`                     |
| `f`    | `notes.txt`                    |
| `F`    | `notes.txt`                    |
| `b`    | `notes`                        |
| `B`    | `../alice/notes`               |
| `e`    | `txt`                          |
| `E`    | `.txt`                         |


Normalized path `p` is constructed using the following rules:

- On Windows, all `/` separators are converted to `\\`.
- Consecutive path separators are collapsed into one.
- Non-root trailing path separator is removed.
- Unnecessary current directory `.` components are removed.
- Parent directory `..` components are resolved where possible.
- Initial `..` components in an absolute path are dropped.
- Initial `..` components in a relative path are kept.
- Empty path is resolved to `.` (current directory).

| Input     | Output |   | Input     | Output |
| --------- |------- | - | --------- |------- |
| *(empty)* | `.`    |   | `/`       | `/`    |
| `.`       | `.`    |   | `/.`      | `/`    |
| `..`      | `..`   |   | `/..`     | `/`    |
| `a/`      | `a`    |   | `/a/`     | `/a`   |
| `a//`     | `a`    |   | `/a//`    | `/a`   |
| `a/.`     | `a`    |   | `/a/.`    | `/a`   |
| `a/..`    | `.`    |   | `/a/..`   | `/`    |
| `./a`     | `a`    |   | `/./a`    | `/a`   |
| `../a`    | `../a` |   | `/../a`   | `/a`   |
| `a//b`    | `a/b`  |   | `/a//b`   | `/a/b` |
| `a/./b`   | `a/b`  |   | `/a/./b`  | `/a/b` |
| `a/../b`  | `b`    |   | `/a/../b` | `/b`   |

Canonical path `P` works similarly to `p` but has some differences:

- Evaluation will fail for a non-existent path.
- Result will always be an absolute path.
- If path is a symbolic link, it will be resolved.

Parent directory `d` might give a different result than `D` which removes last name of a path.
Similarly, file name `f` might not be the same as last name `F` which is a complement of `D`.
 
| Input     | `{d}`   | `{D}`     | `{f}`     | `{F}`     |
| --------- | ------- | --------- | ----------| ----------|
| `/`       | `/`     | `/`       | *(empty)* | *(empty)* |
| `/a`      | `/`     | `/`       | `a`       | `a`       |
| `a/b`     | `a`     | `a`       | `b`       | `b`       |
| `a`       | `.`     | *(empty)* | `a`       | `a`       |
| `..`      | `../..` | *(empty)* | *(empty)* | `..`      |
| `.`       | `./..`  | *(empty)* | *(empty)* | `.`       |
| *(empty)* | `..`    | *(empty)* | *(empty)* | *(empty)* |

### :ab: Substring filters

| Filter | Description                                       |
| ------ | ------------------------------------------------- |
| `nA-B` | Substring from index `A` to `B`.<br/>Indices start from 1 and are both inclusive. |
| `nA-`  | Substring from index `A` to end.                  |
| `nA`   | Character at index `A`.<br/>Equivalent to `nA-A`. |
| `N`    | Same as `n` but with backward indexing.           |

Examples:

| Input   |  Filter | Output |
| ------- | ------- | ------ |
| `abcde` |  `n2-3` | `bc`   |
| `abcde` |  `N2-3` | `cd`   |
| `abcde` |  `n2-`  | `bcde` |
| `abcde` |  `N2-`  | `abcd` |
| `abcde` |  `n2`   | `b`    |
| `abcde` |  `N2`   | `d`    |

### :mag: Replace filters

| Filter  | Description                                             |
| ------- | ------------------------------------------------------- |
| `r:X:Y` | Replace first occurrence of `X` with `Y`.<br/>Any other character than `:` can be also used as a delimiter. |
| `r:X`   | Remove first occurrence of `X`.<br>Equivalent to `r:X:` |
| `R`     | Same as `r` but replaces/removes all occurrences.       |
| `?D`    | Replace empty value with `D`.                           |

Examples:

| Input      |  Filter    | Output  |
| ---------- | ---------- | ------- |
| `ab_ab`    |  `r:ab:xy` | `xy_ab` |
| `ab_ab`    |  `R:ab:xy` | `xy_xy` |
| `ab_ab`    |  `r:ab`    | `_ab`   |
| `ab_ab`    |  `R:ab`    | `_`     |
| `abc`      |  `?def`    | `abc`   |
| *(empty)*  |  `?def`    | `def`   |


### :asterisk: Regex filters

| Filter        | Description                                      |
| ------------- | ------------------------------------------------ |
| `mE`          | Match of regular expression `E`.                 |
| `s:X:Y`       | Replace first match of regular expression `X` with `Y`.<br/>`Y` can reference capture groups from `X` using `$1`, `$2`, ...<br/>Any other character than `:` can be also used as a delimiter. |
| `s:X`         | Remove first match of regular expression `X`.<br/>Equivalent to `s:X:`. |
| `S`           | Same as `s` but replaces/removes all matches.    |
| `1`, `2`, ... | Capture group of an external regular expression. |

Examples:

| Input     |  Filter            | Output  |
| --------- | -------------------| ------- |
| `12_34`   |  `m\d+`            | `12`    |
| `12_34`   |  `s:\d+:x`         | `x_34`  |
| `12_34`   |  `S:\d+:x`         | `x_x`   |
| `12_34`   |  `s:(\d)(\d):$2$1` | `21_34` |
| `12_34`   |  `S:(\d)(\d):$2$1` | `21_43` |

- Use `-e, --regex` / `-E, --regex-filename` option to define an external regular expression.
- Option `-e, --regex` matches regex against each input value.
- Option `-E, --regex-filename` matches regex against *filename component* of each input value.

```bash
echo 'a/b.c' | rew -e '([a-z])' '{1}' # Will print 'a'
echo 'a/b.c' | rew -E '([a-z])' '{1}' # Will print 'b'
```

### :art: Format filters

| Filter | Description                            |
| ------ | -------------------------------------- |
| `t`    | Trim white-spaces from both sides.     |
| `u`    | Convert to uppercase.                  |
| `l`    | Convert to lowercase.                  |
| `i`    | Convert non-ASCII characters to ASCII. |
| `I`    | Remove non-ASCII characters.           |
| `<<M`  | Left pad with mask `M`.                |
| `<N:M` | Left pad with `N` times repeated mask `M`.<br/>Any other non-digit than `:` can be also used as a delimiter. |
| `>>M`  | Right pad with mask `M`.               |
| `>N:M` | Right pad with `N` times repeated mask `M`.<br/>Any other non-digit than `:` can be also used as a delimiter. |

Examples:

| Input      |  Filter    | Output   |
| ---------- | ---------- | -------- |
| `..a..b..` | `t`        | `a..b` *(dots are white-spaces)* |
| `aBčĎ`     | `u`        | `ABČĎ`   |
| `aBčĎ`     | `l`        | `abčď`   |
| `aBčĎ`     | `a`        | `aBcD`   |
| `aBčĎ`     | `A`        | `aB`     |
| `abc`      | `<<123456` | `124abc` |
| `abc`      | `>>123456` | `abc456` |
| `abc`      | `<3:XY`    | `XYXabc` |
| `abc`      | `>3:XY`    | `abcYXY` |

### :infinity: Generators

| Filter | Description                                        |
| ------ | -------------------------------------------------- |
| `*N:V` | Repeat `N` times `V`.<br/>Any other non-digit than `:` can be also used as a delimiter. |
| `c`    | Local counter                                      |
| `C`    | Global counter                                     |
| `uA-B` | Random number from interval \[`A`, `B`]            |
| `uA-`  | Random number from interval \[`A`, `2`<sup>`64`</sup>) |
| `u`    | Random number from interval \[`0`, `2`<sup>`64`</sup>) |
| `U`    | Random UUID                                        |

Examples:

| Filter  | Output                                            |
| ------- | ------------------------------------------------- |
| `*3:ab` | `ababab`                                          |
| `c`     | *(see below)*                                     |
| `C`     | *(see below)*                                     |
| `u0-99` | *(random number between 0-99)*                     |
| `U`     | `5eefc76d-0ca1-4631-8fd0-62eeb401c432` *(random)* |

- Global counter `C` is incremented for every input value.
- Local counter `c` is incremented per parent directory (assuming input value is a path).
- Both counters start at 1 and are incremented by 1.

| Input | Global counter | Local counter |
| ----- | -------------- | ------------- |
| `a/x` | 1              | 1             |
| `a/y` | 2              | 2             |
| `b/x` | 3              | 1             |
| `b/y` | 4              | 2             |

- Use `-c, --local-counter` option to change local counter configuration.
- Use `-C, --global-counter` option to change global counter configuration.

```bash
rew -c0   '{c}' # Start from 0, increment by 1
rew -c2:3 '{c}' # Start from 2, increment by 3
```

## :speech_balloon: Output

By default, results are printed as lines to standard output.

- Use `-Z, --print-nul` flag to print results delimited by NUL character.
- Use `-R, --print-raw` flag to print results without a delimiter.
- Use `-D, --print` options to print results delimited by a specific string.
- Use `-T, --no-trailing-delimiter` flag to not print final delimiter at the end of output.

```bash
rew    '{B}' | xargs    mkdir -p # Pass extracted directories to mkdir command
rew -Z '{B}' | xargs -0 mkdir -p # Use NUL delimiter in case paths contain newlines
rew -D$'\r\n'                    # Convert newlines to CR+LF using custom output delimiter
rew -R '{}#r#n'                  # Same thing but output delimiter is inside pattern
rew -TD+ '{}' a b c              # Join input values to string "a+b+c"
```

Apart from this (standard) mode, there are also two other output modes.

### :robot: Diff mode

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

### :rose: Pretty mode

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

## :microscope: Comparison with similar tools

### `rew` vs `rename` / `prename`

- Unlike `rename`, `rew` can read input paths directly from standard input.
  Use of `xargs` to pass output of `find` or [`fd`][fd] is not needed.
- Unlike `rename`, `rew` is only a text-processing tool and it is unable to rename files.
  You have to use accompanying `mvb` / `cpb` utilities, or you can generate executable shell code.

```bash
find -name '*.jpeg' | xargs rename .jpeg .jpg      # Rename *.jpeg files to *.jpg
find -name '*.jpeg' | rew '{B}.jpg' -b | mvb       # Same thing using rew + mvb
find -name '*.jpeg' | rew 'mv "{}" "{B}.jpg"' | sh # Same thing using rew + mv + sh
```

### `rew` vs `sed` / [`sd`][sd]

- Like `sed` or [`sd`][sd], `rew` is able to replace text using a regular expression.

```bash
echo "foo 123 bar" | sed -E 's/[^0-9]*([0-9]+).*/\1/' # Extract first number using sed
echo "foo 123 bar" | sd '\D*(\d+).*' '$1'    # Same thing using sd
echo "Foo 123 Bar" | rew '{s:\D*(\d+).*:$1}' # Same thing using rew (regex replace filter)
echo "Foo 123 Bar" | rew -e'(\d+)' '{1}'     # Same thing using rew (external regex)
echo "Foo 123 Bar" | rew '{m\d+}'            # Same thing using rew (regex match filter)
```

## :rocket: Examples

> :information_source:
> Use `rew --explain <pattern>` to print detailed explanation what a certain pattern does.

Print contents of working directory as absolute paths.

```bash
rew '{a}' *    # Paths are passed as arguments, wildcard expansion is done by shell
ls | rew '{a}' # Paths are read from standard input
```

Rename all `*.jpeg` files to `*.jpg`.

```bash
find -name '*.jpeg' | rew -b '{B}.jpg' | mvb -v
```

Same thing but we use `rew` to generate executable shell code.

```bash
find -name '*.jpeg' | rew 'mv -v "{}" "{B}.jpg"' | sh
```

Make backup copy of each `*.txt` file with `.txt.bak` extension in the same directory.

```bash
find -name '*.txt'  | rew -b '{}.bak'  | cpb -v
```

Copy `*.txt` files (keep directory structure) to the `~/Backup` directory.

```bash
find -name '*.txt'  | rew -b "$HOME/Backup/{p}"  | cpb -v
```

Copy `*.txt` files (flatten directory structure) to the `~/Backup` directory.

```bash
find -name '*.txt'  | rew -b "$HOME/Backup/{f}"  | cpb -v
```

Same thing but we append randomly generated suffix after base name to avoid name collisions.

```bash
find -name '*.txt'  | rew -b "$HOME/Backup/{b}_{U}.{e}"  | cpb -v
```

Flatten directory structure `./dir/subdir/` to `./dir_subdir/`.

```bash
find -mindepth 2 -maxdepth 2 -type d | rew -b '{p}_{f}' | mvb -v
```

Normalize base names of files to `file_001`, `file_002`, ...

```bash
find -type f | rew -b '{d}/file_{C|<3:0}{E}' | mvb -v
```

Print the first word of each line with removed diacritics (accents).

```bash
rew '{m\S+|i}' <input.txt
```

Swap the first and second column in a CSV file.

```bash
rew -e'([^:]*):([^:]*):(.*)' '{2}:{1}:{3}' <input.csv >output.csv
```

Same thing but we use regex replace filter.

```bash
rew '{s/([^:]*):([^:]*):(.*)/$2:$1:$3}' <input.csv >output.csv
```

Print `PATH` variable entries as lines.

````bash
echo "$PATH" | rew -d: # PATH entries are delimited by ':'
````

Replace tabs with 4 spaces in a file.

````bash
rew -rR '{R:#t:    }' <input.txt >output.txt # Read/write file content as a whole
````

Normalize line endings in a file to `LF`.

````bash
rew <input.txt >output.txt # LF is the default output delimiter
````

Normalize line endings in a file to `CR+LF`.

````bash
rew -D$'\r\n'   <input.txt >output.txt # CR+LF delimiter using -D option
rew -R '{}#r#n' <input.txt >output.txt # CR+LF delimiter inside pattern
````

## :page_facing_up: License

Rew is licensed under the [MIT license](LICENSE.md).

[build]: https://travis-ci.com/github/jpikl/rew
[build-img]: https://travis-ci.com/jpikl/rew.svg?branch=master
[coverage]: https://codecov.io/gh/jpikl/rew
[coverage-img]: https://codecov.io/gh/jpikl/rew/branch/master/graph/badge.svg
[fd]: https://github.com/sharkdp/fd
[rust-dev]: https://www.rust-lang.org/learn/get-started
[sd]: https://github.com/chmln/sd
