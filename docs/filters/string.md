# 🆎 String filters

## Substring

| Filter  | Description                                       |
| ------- | ------------------------------------------------- |
| `#A-B`  | Substring from index `A` to `B`.<br><small>Indices `A`, `B` start from 1 and are both inclusive.<br>Use `-A` for backward indexing.</small> |
| `#A+L`  | Substring from index `A` of length `L`.           |
| `#A-`   | Substring from index `A` to end.                  |
| `#A`    | Character at index `A`.<br><small>Equivalent to `#A-A`.</small>  |

Examples:

| Input   | Pattern  | Output | | Input   | Pattern   | Output |
| ------- | -------- | ------ |-| ------- | --------- | ------ |
| `abcde` | `{#2-3}` | `bc`   | | `abcde` | `{#-2-3}` | `cd`   |
| `abcde` | `{#2+3}` | `bcd`  | | `abcde` | `{#-2+3}` | `bcd`  |
| `abcde` | `{#2-}`  | `bcde` | | `abcde` | `{#-2-}`  | `abcd` |
| `abcde` | `{#2}`   | `b`    | | `abcde` | `{#-2}`   | `d`    |

## String replace

> ℹ️ See [regex filters](regex) for replacement using a regular expression.

| Filter           | Description                                             |
| ---------------- | ------------------------------------------------------- |
| `r:X:Y`          | Replace first occurrence of `X` with `Y`.<br><small>Any other character than `:` can be also used as a delimiter.</small> |
| `r:X`            | Remove first occurrence of `X`.<br><small>Equivalent to `r:X:`.</small> |
| `R:X:Y`<br>`R:X` | Same as `r` but replaces/removes all occurrences.       |
| `?D`             | Replace empty value with `D`.                           |

Examples:

| Input     |  Pattern    | Output  |
| --------- | ----------- | ------- |
| `ab_ab`   | `{r:ab:xy}` | `xy_ab` |
| `ab_ab`   | `{R:ab:xy}` | `xy_xy` |
| `ab_ab`   | `{r:ab}`    | `_ab`   |
| `ab_ab`   | `{R:ab}`    | `_`     |
| `abc`     | `{?def}`    | `abc`   |
| *(empty)* | `{?def}`    | `def`   |
