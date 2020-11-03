# rew

Rew is a CLI tool that rewrites FS paths according to a pattern.

[![Build status][build-img]][build]
[![Code coverage][coverage-img]][coverage]

:construction: Work in progress... :construction:

## Contents

- [:bulb: What rew does](#bulb-what-rew-does)
- [:inbox_tray: Input](#inbox_tray-input)
- [:pencil: Pattern](#pencil-pattern)
  - [Variables](#variables)
  - [Filters](#filters)
- [:outbox_tray: Output](#outbox_tray-output)

## :bulb: What rew does

1. Reads values from standard input.
2. Rewrites them using provided pattern.
3. Prints results to standard output.

Input values are assumed to be FS paths, however, `rew` is able to process any UTF-8 encoded text.

```text
+------------+       +-------------------+       +-----------+
| STDIN      |  -->  | PATTERN           |  -->  | STDOUT    |
+------------+       +-------------------+       +-----------+
| photo.PNG  |       | img_{C}.{e|l|r:e} |       | img_1.png |
| image.jpeg |       +-------------------+       | img_2.jpg |
| logo.gif   |                                   | img_3.gif |
+------------+                                   +-----------+
```

[build]: https://travis-ci.com/github/jpikl/rew
[build-img]: https://travis-ci.com/jpikl/rew.svg?branch=master
[coverage]: https://codecov.io/gh/jpikl/rew
[coverage-img]: https://codecov.io/gh/jpikl/rew/branch/master/graph/badge.svg

## :inbox_tray: Input

By default, paths are read as lines from standard input.

```bash
find | rew '{f}' # Process paths generated by find command
```

Use `-z, --read-nul` flag to read paths separated by NUL character.

```bash
find -print0 | rew -z '{f}' # When paths contain newlines
```

Use `-r, --read-raw` flag to read whole input as a single value.

```bash
rew -r '{f}' < data.bin # Process file content as a whole
```

Input paths can be also provided as additional arguments.

```bash
rew '{f}' *.txt # Wildcard expansion done by shell
```

## :pencil: Pattern

Pattern is a string describing how to generate output from an input.

Use `--explain` flag to print detailed explanation what a certain pattern does.

```bash
rew --explain 'file_{c|<000}.{e}'
```

By default, pattern characters are directly copied to output.

| Input | Pattern | Output |
| ----- | ------- | ------ |
| *     | `abc`   | `abc`  |

Characters between `{` and `}` form an expression which it is evaluated against input.

| Input      | Pattern   | Output    | Expression description |
| ---------- | --------- | --------- | ---------------------- |
| `file.txt` | `{b}`     | `file`    | Base name              |
| `file.txt` | `new.{e}` | `new.txt` | Extension              |

Expression `{v|f1|f2|...}` is made of a variable `v` and zero or more filters `f1`, `f2`, ..., separated by `|`.

| Input      | Pattern           | Output     | Expression description             |
| ---------- | ----------------- | ---------- | ---------------------------------- |
| `img.JPEG` | `new.{e}`         | `new.JPEG` | Extension                          |
| `img.JPEG` | `new.{e\|l}`      | `new.jpeg` | Extension + Lowercase              |
| `img.JPEG` | `new.{e\|l\|r:e}` | `new.jpg`  | Extension + Lowercase + Remove `e` |

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
rew '{p|R:#t: }'              # Replace tabs by spaces in path
rew '{p|R:\t: }' --escape='\' # Same thing, different escape character
```

### Variables

| Variable      | Description                  |
| ------------- | ---------------------------- |
| `p`           | Path (equal to input value)  |
| `a`           | Absolute path                |
| `A`           | Canonical path               |
| `f`           | File name                    |
| `b`           | Base name                    |
| `e`           | Extension                    |
| `E`           | Extension with dot           |
| `d`           | Parent path                  |
| `D`           | Parent file name             |
| `c`           | Local counter                |
| `C`           | Global counter               |
| `u`           | Randomly generated UUID v4   |
| `1`, `2`, ... | Regex capture group N        |

Let us assume the following directory structure:

```text
/
└── home
    ├── alice
    │   └── docs
    │       └── notes.txt
    |
    └── bob
```

For working directory `/home/bob` and input `../alice/docs/notes.txt`
variables would be evaluated as:

| Variable | Output                             |
| -------- | ---------------------------------- |
| `p`      | `../alice/dir/notes.txt`           |
| `a`      | `/home/bob/../alice/dir/notes.txt` |
| `A`      | `/home/alice/dir/notes.txt`        |
| `f`      | `notes.txt`                        |
| `b`      | `notes`                            |
| `e`      | `txt`                              |
| `E`      | `.txt`                             |
| `d`      | `../alice/docs`                    |
| `D`      | `docs`                             |

#### Counters

- Global counter `C` is incremented for every input.
- Local counter `c` is incremented per directory.

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

#### Regular expressions

- Use `-e, --regex` option to match regular expression against filename.
- Use `-E, --regex-full` option to match regular expression against whole path.
- The matched capture groups can be referenced using `1`, `2`, ...

```bash
rew -e '([0-9]+)' '{1}' # Print the first number in filename
rew -E '([0-9]+)' '{1}' # Print the first number in whole path
```

### Filters

#### Substring filters

| Filter     | Description                                        |
| ---------- | -------------------------------------------------- |
| `nA-B`     | Substring from index `A` to `B`.<br/>Indices start from 1 and are both inclusive. |
| `nA-`      | Substring from index `A` to end.                   |
| `n-B`      | Substring from start to index `B`.                 |
| `nA`       | Character at index `A` (equivalent to `nA-A`)      |
| `N`        | Same as `n` but we are indexing from end to start. |

#### Examples of substring filters

| Input   |  Filter                  | Output  |
| ------- | ------------------------ | ------- |
| `abcde` |  `n2-3`                  | `bc`    |
| `abcde` |  `N2-3`                  | `cd`    |
| `abcde` |  `n2-`                   | `bcde`  |
| `abcde` |  `N2-`                   | `abcd`  |
| `abcde` |  `n-2`                   | `ab`    |
| `abcde` |  `N-2`                   | `de`    |
| `abcde` |  `n2`                    | `b`     |
| `abcde` |  `N2`                    | `d`     |

#### Replace filters

| Filter     | Description                                       |
| ---------- | ------------------------------------------------- |
| `r:X:Y`    | Replace first occurrence of `X` by `Y`.<br/>Any other character than `:` can be also used as a separator. |
| `r:X`      | Remove first occurrence of `X`.                   |
| `s`        | Same as `r` but `X` is an regular expression.<br/>`Y` can reference capture groups from `X` using `$1`, `$2`, ... |
| `R`        | Same as `r` but removes/replaces all occurrences. |
| `S`        | Same as `s` but removes/replaces all occurrences. |
| `?D`       | Replace empty input with D.                       |

#### Examples of replace filters

| Input     |  Filter                  | Output  |
| --------- | ------------------------ | ------- |
| `ab_ab`   |  `r:ab:xy`               | `xy_ab` |
| `ab_ab`   |  `R:ab:xy`               | `xy_xy` |
| `ab_ab`   |  `r:ab`                  | `_ab`   |
| `ab_ab`   |  `R:ab`                  | `_`     |
| `12_34`   |  `s:[0-9]+:x`            | `xx_34` |
| `12_34`   |  `S:[0-9]+:x`            | `xx_xx` |
| `12_34`   |  `s:([0-9])([0-9]):$2$1` | `21_34` |
| `12_34`   |  `S:([0-9])([0-9]):$2$1` | `21_43` |
| `abc`     |  `?def`                  | `abc`   |
| *(empty)* |  `?def`                  | `def`   |

#### Other filters

| Filter     | Description                          |
| ---------- | ------------------------------------ |
| `t`        | Trim white-spaces from bother sides. |
| `u`        | Convert to uppercase.                |
| `l`        | Convert to lowercase.                |
| `a`        | Convert non-ASCII characters ASCII.  |
| `A`        | Remove non-ASCII characters.         |
| `<M`       | Left pad with mask M.                |
| `>M`       | Right pad with mask M.               |

#### Examples of other filters

| Input      |  Filter  | Output  |
| ---------- | -------- | ------- |
| `..a..b..` | `t`      | `a..b` *(dots are white-spaces)* |
| `aBčĎ`     | `u`      | `ABČĎ`  |
| `aBčĎ`     | `l`      | `abčď`  |
| `aBčĎ`     | `a`      | `aBcD`  |
| `aBčĎ`     | `A`      | `aB`    |
| `abc`      | `<12345` | `12abc` |
| `abc`      | `>12345` | `abc45` |

## :outbox_tray: Output

By default, results are printed as lines to standard output.

```bash
rew '{f}' | xargs echo # Pass generated paths to echo command
```

Use `-Z, --print-nul` flag to print results separated by NUL character.

```bash
rew -Z '{f}' | xargs -0 echo # When paths contain newlines
```

Use `-R, --print-raw` flag to print results without any delimiter.

```bash
rew -r '{f};' # Use semicolon as our own delimiter
```

Use `-p, --pretty` flag to print nicely formatted transformations.

```bash
rew -p '{f}' # Output formatted as "src_path -> dst_path"
```

Use `-b, --bulk` flag to print transformations in the following format.

```text
<src_path_1
>dst_path_1
<src_path_2
>dst_path_2
...
<src_path_N
>dst_path_N
```

Such output can be processed by accompanying `mvb`  and `cpb` utilities to perform bulk move/rename of files and directories.

```bash
rew -b '{p}.bak' | cpb # Make backup copy of each file
```

The `-b, --bulk` flag can be combined with `-Z, --print-nul` or `-R, --print-raw`.

```bash
rew -bZ '{p}.bak' | cpb -z # When paths contain newlines
```

Bulk copy can be also achieved using standard `cp` command in less efficient way.

```bash
rew 'cp "{p}" "{p}.bak"' | sh # Generate code and execute it
```
