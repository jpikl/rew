# 🔍 Replace filters

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
