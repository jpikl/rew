# 🎨 Format filters

| Filter | Description                            |
| ------ | -------------------------------------- |
| `t`    | Trim white-spaces from both sides.     |
| `l`    | Convert to lowercase.                  |
| `L`    | Convert to uppercase.                  |
| `i`    | Convert non-ASCII characters to ASCII. |
| `I`    | Remove non-ASCII characters.           |
| `<<M`  | Left pad with mask `M`.                |
| `<N:M` | Left pad with `N` times repeated mask `M`.<br/>Any other non-digit than `:` can be also used as a delimiter. |
| `>>M`  | Right pad with mask `M`.               |
| `>N:M` | Right pad with `N` times repeated mask `M`.<br/>Any other non-digit than `:` can be also used as a delimiter. |

Examples:

| Input      |  Pattern     | Output   |
| ---------- | ------------ | -------- |
| `..a..b..` | `{t}`        | `a..b` *(dots are white-spaces)* |
| `aBčĎ`     | `{u}`        | `ABČĎ`   |
| `aBčĎ`     | `{l}`        | `abčď`   |
| `aBčĎ`     | `{a}`        | `aBcD`   |
| `aBčĎ`     | `{A}`        | `aB`     |
| `abc`      | `{<<123456}` | `123abc` |
| `abc`      | `{>>123456}` | `abc456` |
| `abc`      | `{<3:XY}`    | `XYXabc` |
| `abc`      | `{>3:XY}`    | `abcYXY` |
