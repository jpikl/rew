# 🛤 Path filters

Path filters assume that their input value is a FS path.

## Path components

| Filter | Description      |   | Filter | Description        |
| ------ | ---------------- | - | ------ | ------------------ |
| `d`    | Parent directory |   | `D`    | Remove last name   |
| `f`    | File name        |   | `F`    | Last name          |
| `b`    | Base name        |   | `B`    | Remove extension   |
| `e`    | Extension        |   | `E`    | Extension with dot |

For input value `/home/alice/notes.txt`, filters would evaluate to:

| Pattern      | Output                  |
| ------------ | ----------------------- |
| `{}`         | `/home/alice/notes.txt` |
| `{d}`, `{D}` | `/home/alice`           |
| `{f}`, `{F}` | `notes.txt`             |
| `{b}`        | `notes`                 |
| `{B}`        | `/home/alice/notes`     |
| `{e}`        | `txt`                   |
| `{E}`        | `.txt`                  |

Parent directory `d` might give a different result than `D` which removes last name of a path.
Similarly, file name `f` might not be the same as last name `F` which is a complement of `D`.

| Input     | `{d}`   | `{D}`     | `{f}`     | `{F}`     |
| --------- | ------- | --------- | ----------| ----------|
| `/`       | `/`     | `/`       | *(empty)* | *(empty)* |
| `/a`      | `/`     | `/`       | `a`       | `a`       |
| `a/b`     | `a`     | `a`       | `b`       | `b`       |
| `a`       | `.`     | *(empty)* | `a`       | `a`       |
| `.`       | `./..`  | *(empty)* | *(empty)* | `.`       |
| `..`      | `../..` | *(empty)* | *(empty)* | `..`      |
| *(empty)* | `..`    | *(empty)* | *(empty)* | *(empty)* |

Extension with dot `E` can be useful when dealing with files with no extension.

| Input     | `new.{e}` | `new{E}`  |
| --------- | --------- | --------- |
| `old.txt` | `new.txt` | `new.txt` |
| `old`     | `new.`    | `new`     |

## Absolute and relative paths

| Filter | Description       |
| ------ | ----------------- |
| `w`    | Working directory |
| `a`    | Absolute path     |
| `A`    | Relative path     |

Absolute path `a` and relative path `A` are both resolved against working directory `w`.

| `{w}`         | Input       | `{a}`       | `{A}`    |
| ------------- | ----------- | ----------- | -------- |
| `/home/alice` | `/home/bob` | `/home/bob` | `../bob` |
| `/home/alice` | `../bob`    | `/home/bob` | `../bob` |

By default, working directory `w` is set to your current working directory.
You can change that using the `-w, --working-directory` option.
`w` filter will always output an absolute path, even if you set a relative one using the `-w` option.

```bash
rew -w '/home/alice' '{w}' # Absolute path
rew -w '../alice'    '{w}' # Relative to your current working directory
```

## Path normalization

| Filter | Description      |
| ------ | ---------------- |
| `p`    | Normalized path  |
| `P`    | Canonical path   |

Normalized path `p` is constructed using the following rules:

- On Windows, all `/` separators are converted to `\`.
- Consecutive directory separators are collapsed into one.
- Non-root trailing directory separator is removed.
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

## Directory separator

| Filter | Description                         |
| ------ | ----------------------------------- |
| `z`    | Ensure trailing directory separator |
| `Z`    | Remove trailing directory separator |

Directory separator filters `z` and `Z` can be useful when dealing with root and unnormalized paths.

| Input  | `{}b` | `{}/b` | `{z}b` | `{Z}/b` |
| ------ | ----- | -------| ------ | ------- |
| `/`    | `/b`  | `//b`  | `/b`   | `/b`    |
| `a`    | `ab`  | `a/b`  | `a/b`  | `a/b`   |
| `a/`   | `a/b` | `a//b` | `a/b`  | `a/b`   |
