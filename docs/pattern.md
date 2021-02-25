# ✏️ Pattern

Pattern is a string describing how to generate output from an input.

Use `--explain` flag to print detailed explanation what a certain pattern does.

```bash
rew --explain 'file_{c|<3:0}.{e}'
```

When no pattern is provided as an argument, the default pattern `{}` is used.

```bash
rew '{}' # The default pattern
rew      # Also uses the default pattern
```

## Syntax

By default, pattern characters are directly copied to output.

| Input   | Pattern | Output |
| ------- | ------- | ------ |
| *(any)* | `abc`   | `abc`  |

Characters `{` and `}` form an expression which is evaluated and replaced in output.
Empty expression `{}` evaluates directly to input value.

| Input   | Pattern      | Output          |
| ------- | ------------ | --------------- |
| `world` | `{}`         | `world`         |
| `world` | `Hello, {}!` | `Hello, world!` |

Expression may contain one or more filters, delimited by `|`.
Filters are consecutively applied on input value.

| Input      | Pattern         | Output     | Description                        |
| ---------- | --------------- | ---------- | ---------------------------------- |
| `old.JPEG` | `new.{e}`       | `new.JPEG` | Extension                          |
| `old.JPEG` | `new.{e|l}`     | `new.jpeg` | Extension + Lowercase              |
| `old.JPEG` | `new.{e|l|r:e}` | `new.jpg`  | Extension + Lowercase + Remove `e` |

## Filters

Filters are categorized into the following groups.

- [🛤 Path filters](filters/path.md)
- [🆎 Substring filters](filters/substr.md)
- [🔍 Replace filters](filters/replace.md)
- [⭐️ Regex filters](filters/regex.md)
- [🎨 Format filters](filters/format.md)
- [🏭 Generators](filters/generators.md)

## Escaping

Character `%` starts an escape sequence.

| Sequence | Description                |
| -------- |--------------------------- |
| `%/`     | System directory separator<br>`\` on Windows<br>`/` everywhere else |
| `%n`     | New line                   |
| `%r`     | Carriage return            |
| `%t`     | Horizontal tab             |
| `%0`     | Null                       |
| `%{`     | Escaped `{`                |
| `%|`     | Escaped `|`                |
| `%}`     | Escaped `{`                |
| `%%`     | Escaped `%`                |

Use `--escape` option to set a different escape character.

```bash
rew '{R:%t: }'              # Replace tabs with spaces
rew '{R:\t: }' --escape='\' # Same thing, different escape character
```
