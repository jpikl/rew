# ⭐️ Regex filters

| Filter   | Description                                   |
| -------- | --------------------------------------------- |
| `=E`     | Match of a regular expression `E`.            |
| `s:X:Y`  | Replace first match of a regular expression `X` with `Y`.<br/>`Y` can reference capture groups from `X` using `$0`, `$1`, `$2`, ...<br/>Any other character than `:` can be also used as a delimiter. |
| `s:X`    | Remove first match of a regular expression `X`.<br/>Equivalent to `s:X:`. |
| `S`      | Same as `s` but replaces/removes all matches. |
| `@:X1:Y1:...:Xn:Yn:D` | Regular expression switch.<br/>Output `Yi` for first `Xi` that matches input.<br/>Output `D` when there is no match.<br/>`Yi` can reference capture groups from `Xi` using `$0`, `$1`, `$2`, ...<br/>Any other character than `:` can be also used as a delimiter.
| `$0`, `$1`, `$2`, ... | Capture group of a global regular expression. |

Examples:

| Input     | Pattern             | Output  |
| --------- | --------------------| ------- |
| `12_34`   | `{=\d+}`            | `12`    |
| `12_34`   | `{s:\d+:x}`         | `x_34`  |
| `12_34`   | `{S:\d+:x}`         | `x_x`   |
| `12_34`   | `{s:(\d)(\d):$2$1}` | `21_34` |
| `12_34`   | `{S:(\d)(\d):$2$1}` | `21_43` |
| *(any)* | `{@:def}`                                   | `def`              |
| `ab`    | `{@:^[a-z]+$:lower:^[A-Z]+$:upper:mixed}`   | `lower`            |
| `AB`    | `{@:^[a-z]+$:lower:^[A-Z]+$:upper:mixed}`   | `upper`            |
| `Ab`    | `{@:^[a-z]+$:lower:^[A-Z]+$:upper:mixed}`   | `mixed`            |
| `a=b`   | `{@/(.+)=(.*)/key: $1, value: $2/invalid}`  | `key: a, value: b` |
| `ab`    | `{@/(.+)=(.*)/key: $1, value: $2/invalid}`  | `invalid`          |

- Use `-e, --regex` or `-E, --regex-filename` option to define a global regular expression.
- Option `-e, --regex` matches regex against each input value.
- Option `-E, --regex-filename` matches regex against *filename component* of each input value.

```bash
echo 'a/b.c' | rew -e '([a-z])' '{$1}' # Will print 'a'
echo 'a/b.c' | rew -E '([a-z])' '{$1}' # Will print 'b'
```
