# 🆎 Substring filters

| Filter  | Description                                       |
| ------- | ------------------------------------------------- |
| `#A-B`  | Substring from index `A` to `B`.<br/>Indices start from 1 and are both inclusive. |
| `#A+L`  | Substring from index `A` of length `L`.         |
| `#A-`   | Substring from index `A` to end.                  |
| `#A`    | Character at index `A`.<br/>Equivalent to `nA-A`. |
| `#-A-B` | Same as `#A-B` but with backward indexing. |
| `#-A+L` | Same as `#A+L` but with backward indexing. |
| `#-A-`  | Same as `#A-` but with backward indexing. |
| `#-A`   | Same as `#A` but with backward indexing. |

Examples:

| Input   | Pattern   | Output |
| ------- | --------- | ------ |
| `abcde` | `{#2-3}`  | `bc`   |
| `abcde` | `{#2+1}`  | `bc`   |
| `abcde` | `{#2-}`   | `bcde` |
| `abcde` | `{#2}`    | `b`    |
| `abcde` | `{#-2-3}` | `cd`   |
| `abcde` | `{#-2+1}` | `cd`   |
| `abcde` | `{#-2-}`  | `abcd` |
| `abcde` | `{#-2}`   | `d`    |
