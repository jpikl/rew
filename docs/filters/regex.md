# ⭐️ Regex filters

## Regex replace

| Filter           | Description                                   |
| ---------------- | --------------------------------------------- |
| `s:X:Y`          | Replace first match of a regular expression `X` with `Y`.<br>*`Y` can reference capture groups from `X` using `$0`, `$1`, `$2`, ...<br>Any other character than `:` can be also used as a delimiter.* |
| `s:X`            | Remove first match of a regular expression `X`.<br>*Equivalent to `s:X:`.* |
| `S:X:Y`<br>`S:X` | Same as `s` but replaces/removes all matches. |

Examples:

| Input     | Pattern             | Output  |
| --------- | --------------------| ------- |
| `12_34`   | `{s:\d+:x}`         | `x_34`  |
| `12_34`   | `{S:\d+:x}`         | `x_x`   |
| `12_34`   | `{s:(\d)(\d):$2$1}` | `21_34` |
| `12_34`   | `{S:(\d)(\d):$2$1}` | `21_43` |

## Regex match

| Filter | Description                        |
| ------ | ---------------------------------- |
| `=E`   | Match of a regular expression `E`. |

Examples:

| Input     | Pattern  | Output  |
| --------- | -------- | ------- |
| `12_34`   | `{=\d+}` | `12`    |

## Regex switch

| Filter                | Description                                                       |
| --------------------- | ----------------------------------------------------------------- |
| `@:X:Y`               | Output `Y` if input matches regular expression `X`<br>Output nothing when there is no match..<br>*`Y` can reference capture groups from `X` using `$0`, `$1`, `$2`, ...<br>Any other character than `:` can be also used as a delimiter.* |
| `@:X:Y:D`             | Output `Y` if input matches regular expression `X`.<br>Output `D` when there is no match. |
| `@:X1:Y1:...:Xn:Yn:D` | Output `Yi` for first regular expression `Xi` that matches input.<br>Output `D` when there is no match. |
| `@:D`                 | A switch without any `Xi`/`Yi` cases which will always output `D`. |

Examples:

| Input   | Pattern                                     | Output             |
| ------- | --------------------------------------------| ------------------ |
| *(any)* | `{@:def}`                                   | `def`              |
| `12`    | `{@:^\d+$:number}`                          | `number`           |
| `1x`    | `{@:^\d+$:number}`                          | *(empty)*          |
| `12`    | `{@:^\d+$:number:string}`                   | `number`           |
| `1x`    | `{@:^\d+$:number:string}`                   | `string`           |
| `ab`    | `{@:^[a-z]+$:lower:^[A-Z]+$:upper:mixed}`   | `lower`            |
| `AB`    | `{@:^[a-z]+$:lower:^[A-Z]+$:upper:mixed}`   | `upper`            |
| `Ab`    | `{@:^[a-z]+$:lower:^[A-Z]+$:upper:mixed}`   | `mixed`            |
| `a=b`   | `{@/(.+)=(.*)/key: $1, value: $2/invalid}`  | `key: a, value: b` |
| `ab`    | `{@/(.+)=(.*)/key: $1, value: $2/invalid}`  | `invalid`          |

## Global regex

| Filter                | Description                                   |
| --------------------- | --------------------------------------------- |
| `$0`, `$1`, `$2`, ... | Capture group of a global regular expression. |

- Use `-e, --regex` or `-E, --regex-filename` option to define a global regular expression.
- Option `-e, --regex` matches regex against each input value.
- Option `-E, --regex-filename` matches regex against *filename component* of each input value.

```bash
echo 'a/b.c' | rew -e '([a-z])' '{$1}' # Will print 'a'
echo 'a/b.c' | rew -E '([a-z])' '{$1}' # Will print 'b'
```
