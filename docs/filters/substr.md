# 🆎 Substring filters

| Filter | Description                                       |
| ------ | ------------------------------------------------- |
| `nA-B` | Substring from index `A` to `B`.<br/>Indices start from 1 and are both inclusive. |
| `nA-`  | Substring from index `A` to end.                  |
| `nA`   | Character at index `A`.<br/>Equivalent to `nA-A`. |
| `N`    | Same as `n` but with backward indexing.           |

Examples:

| Input   | Pattern  | Output |
| ------- | -------- | ------ |
| `abcde` | `{n2-3}` | `bc`   |
| `abcde` | `{N2-3}` | `cd`   |
| `abcde` | `{n2-}`  | `bcde` |
| `abcde` | `{N2-}`  | `abcd` |
| `abcde` | `{n2}`   | `b`    |
| `abcde` | `{N2}`   | `d`    |
