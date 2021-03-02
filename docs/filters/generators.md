# 🏭 Generators

Unlike other filters, generator output is not produced from its input.
However, it is still possible (although meaningless) to pipe input into a generator.

| Filter | Description                            |
| ------ | -------------------------------------- |
| `*N:V` | Repeat `N` times `V`.<br>Any other character than `:` can be also used as a delimiter. |
| `c`    | Local counter                          |
| `C`    | Global counter                         |
| `uA-B` | Random 64-bit number (`A` ≤ `u` ≤ `B`) |
| `uA-`  | Random 64-bit number (`A` ≤ `u`)       |
| `u`    | Random 64-bit number                   |
| `U`    | Random UUID                            |

Examples:

| Pattern   | Output                                            |
| --------- | ------------------------------------------------- |
| `{*3:ab}` | `ababab`                                          |
| `{c}`     | *(see below)*                                     |
| `{C}`     | *(see below)*                                     |
| `{u0-99}` | *(random number between 0 and 99)*                |
| `{U}`     | `5eefc76d-0ca1-4631-8fd0-62eeb401c432` *(random)* |

- Global counter `C` is incremented for every input value.
- Local counter `c` is incremented per parent directory (assuming input value is a FS path).
- Both counters start at 1 and are incremented by 1.

| Input | Global counter | Local counter |
| ----- | -------------- | ------------- |
| `A/1` | 1              | 1             |
| `A/2` | 2              | 2             |
| `B/1` | 3              | 1             |
| `B/2` | 4              | 2             |

- Use `-c, --local-counter` option to change local counter configuration.
- Use `-C, --global-counter` option to change global counter configuration.

```bash
rew -c0   '{c}' # Start from 0, increment by 1
rew -c2:3 '{c}' # Start from 2, increment by 3
```
