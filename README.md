# rew

Rew is a CLI tool that rewrites FS paths according to a pattern.

[![Build status][build-img]][build]
[![Code coverage][coverage-img]][coverage]

:construction: Work in progress... :construction:

## Contents

- [:bulb: What rew does](#bulb-what-rew-does)
- [:keyboard: Input](#keyboard-input)
- [:pencil: Pattern](#pencil-pattern)
  - [Path filters](#path-filters)
  - [Substring filters](#substring-filters)
  - [Replace filters](#replace-filters)
  - [Regex filters](#regex-filters)
  - [Format filters](#format-filters)
  - [Generators](#generators)
- [:speech_balloon: Output](#speech_balloon-output)

## :bulb: What rew does

1. Reads values from standard input.
2. Rewrites them using provided pattern.
3. Prints results to standard output.

Input values are assumed to be FS paths, however, `rew` is able to process any UTF-8 encoded text.

![What rew does](images/diagram.png)

## :keyboard: Input

By default, values are read as lines from standard input. 
`CR+LF` or `LF` is auto-detected as a delimiter, independent of platform.

```bash
find | rew '{a}' # Process find output and print absolute paths
```

Use `-z, --read-nul` flag to read values delimited by NUL character.

```bash
find -print0 | rew -z '{a}' # Paths may contain newlines
```

Use `-d, --read` option to read values delimited by a specific character.

```bash
echo "$PATH" | rew -d ':' # Split paths delimited by colon
```

Use `-r, --read-raw` flag to read whole input into memory as a single value.

```bash
rew -r '{R:#r#n:#n}' <input.txt >output.txt # Replace CR+LF with LF in file
```

Input values can be also provided as additional arguments.

```bash
rew '{a}' *.txt # Wildcard expansion done by shell
```

## :pencil: Pattern

Pattern is a string describing how to generate output from an input.

Use `--explain` flag to print detailed explanation what a certain pattern does.

```bash
rew --explain 'file_{c|<00}.{e}'
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

### Path filters

| Filter | Description         |
| ------ | ------------------- |
| `a`    | Absolute path       |
| `A`    | Canonical path      |
| `p`    | Parent path         |
| `f`    | File name           |
| `b`    | Base name           |
| `e`    | Extension           |
| `E`    | Extension with dot  |

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
| `A`    | `/home/alice/notes.txt`        |
| `p`    | `../alice`                     |
| `f`    | `notes.txt`                    |
| `b`    | `notes`                        |
| `e`    | `txt`                          |
| `E`    | `.txt`                         |

###  Substring filters

| Filter | Description                                       |
| ------ | ------------------------------------------------- |
| `nA-B` | Substring from index `A` to `B`.<br/>Indices start from 1 and are both inclusive. |
| `nA-`  | Substring from index `A` to end.                  |
| `n-B`  | Substring from start to index `B`.                |
| `nA`   | Character at index `A`.<br/>Equivalent to `nA-A`. |
| `N`    | Same as `n` but with backward indexing.           |

Examples:

| Input   |  Filter | Output |
| ------- | ------- | ------ |
| `abcde` |  `n2-3` | `bc`   |
| `abcde` |  `N2-3` | `cd`   |
| `abcde` |  `n2-`  | `bcde` |
| `abcde` |  `N2-`  | `abcd` |
| `abcde` |  `n-2`  | `ab`   |
| `abcde` |  `N-2`  | `de`   |
| `abcde` |  `n2`   | `b`    |
| `abcde` |  `N2`   | `d`    |

###  Replace filters

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


###  Regex filters

| Filter  | Description                                   |
| ------- | --------------------------------------------- |
| `mE`    | Match of regular expression `E`.              |
| `s:X:Y` | Replace first match of regular expression `X` with `Y`.<br/>`Y` can reference capture groups from `X` using `$1`, `$2`, ...<br/>Any other character than `:` can be also used as a delimiter. |
| `s:X`   | Remove first match of regular expression `X`.<br/>Equivalent to `s:X:`. |
| `S`     | Same as `s` but replaces/removes all matches. |

Examples:

| Input     |  Filter                  | Output  |
| --------- | ------------------------ | ------- |
| `12_34`   |  `m[0-9]+`               | `12`    |
| `12_34`   |  `s:[0-9]+:x`            | `x_34`  |
| `12_34`   |  `S:[0-9]+:x`            | `x_x`   |
| `12_34`   |  `s:([0-9])([0-9]):$2$1` | `21_34` |
| `12_34`   |  `S:([0-9])([0-9]):$2$1` | `21_43` |

###  Format filters

| Filter  | Description                            |
| ------- | -------------------------------------- |
| `t`     | Trim white-spaces from both sides.     |
| `u`     | Convert to uppercase.                  |
| `l`     | Convert to lowercase.                  |
| `a`     | Convert non-ASCII characters to ASCII. |
| `A`     | Remove non-ASCII characters.           |
| `<M`    | Left pad with mask `M`.                |
| `>M`    | Right pad with mask `M`.               |

Examples:

| Input      |  Filter  | Output  |
| ---------- | -------- | ------- |
| `..a..b..` | `t`      | `a..b` *(dots are white-spaces)* |
| `aBčĎ`     | `u`      | `ABČĎ`  |
| `aBčĎ`     | `l`      | `abčď`  |
| `aBčĎ`     | `a`      | `aBcD`  |
| `aBčĎ`     | `A`      | `aB`    |
| `abc`      | `<12345` | `12abc` |
| `abc`      | `>12345` | `abc45` |

### Generators

| Filter | Description             |
| ------ | ----------------------- |
| `c`    | Local counter.          |
| `c`    | Global counter.         |
| `u`    | Randomly generated UUID |

- Global counter `C` is incremented for every input value.
- Local counter `c` is incremented per parent directory (assuming input value is a path).

| Input | Global counter | Local counter |
| ----- | -------------- | ------------- |
| `a/x` | 1              | 1             |
| `a/y` | 2              | 2             |
| `b/x` | 3              | 1             |
| `b/y` | 4              | 2             |

- Use `--gc-init, --gc-step` options to set initial/step value for global counter.
- Use `--lc-init, --lc-step` options to set initial/step value for local counter.

```bash
rew --gc-init=0 --gc-step=2 '{C}' # Start from 0, increment by 2
rew --lc-init=1 --lc-step=3 '{c}' # Start from 1, increment by 3
```

## :speech_balloon: Output

By default, results are printed as lines to standard output.

```bash
rew '{p}' | xargs mkdir -p # Pass extracted directories to mkdir command
```

Use `-Z, --print-nul` flag to print results delimited by NUL character.

```bash
rew -Z '{p}' | xargs -0 mkdir -p # Paths may contain newlines
```

Use `-R, --print-raw` flag to print results without any delimiter.

```bash
rew -R '{}:' # Print comma separated values
```

Use `-p, --pretty` flag to print nicely formatted transformations.

```bash
rew -p '{f}' # Output formatted as "input_value -> output_value"
```

Use `-b, --bulk` flag to print transformations in the following format.

```text
<input_value_1
>output_value_1
<input_value_2
>output_value_2
...
<input_value_N
>output_value_N
```

Such output can be processed by accompanying `mvb`  and `cpb` utilities to perform bulk move/copy of files and directories.

```bash
rew -b '{}.bak' | cpb # Make backup copy of each file
```

The `-b, --bulk` flag can be combined with `-Z, --print-nul` or `-R, --print-raw`.

```bash
rew -bZ '{}.bak' | cpb -z # Paths may contain newlines
```

Bulk move/copy can be also achieved using standard `mv`/`cp` utilities in less efficient way.

```bash
rew 'cp "{}" "{}.bak"' | sh # Generate shell code and execute it
```

[build]: https://travis-ci.com/github/jpikl/rew
[build-img]: https://travis-ci.com/jpikl/rew.svg?branch=master
[coverage]: https://codecov.io/gh/jpikl/rew
[coverage-img]: https://codecov.io/gh/jpikl/rew/branch/master/graph/badge.svg
