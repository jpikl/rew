# ⭐️ Regex filters

| Filter        | Description                                      |
| ------------- | ------------------------------------------------ |
| `=E`          | Match of a regular expression `E`.               |
| `s:X:Y`       | Replace first match of a regular expression `X` with `Y`.<br/>`Y` can reference capture groups from `X` using `$1`, `$2`, ...<br/>Any other character than `:` can be also used as a delimiter. |
| `s:X`         | Remove first match of a regular expression `X`.<br/>Equivalent to `s:X:`. |
| `S`           | Same as `s` but replaces/removes all matches.    |
| `1`, `2`, ... | Capture group of an external regular expression. |

Examples:

| Input     |  Pattern             | Output  |
| --------- | ---------------------| ------- |
| `12_34`   |  `{=\d+}`            | `12`    |
| `12_34`   |  `{s:\d+:x}`         | `x_34`  |
| `12_34`   |  `{S:\d+:x}`         | `x_x`   |
| `12_34`   |  `{s:(\d)(\d):$2$1}` | `21_34` |
| `12_34`   |  `{S:(\d)(\d):$2$1}` | `21_43` |

- Use `-e, --regex` / `-E, --regex-filename` option to define an external regular expression.
- Option `-e, --regex` matches regex against each input value.
- Option `-E, --regex-filename` matches regex against *filename component* of each input value.

```bash
echo 'a/b.c' | rew -e '([a-z])' '{1}' # Will print 'a'
echo 'a/b.c' | rew -E '([a-z])' '{1}' # Will print 'b'
```
