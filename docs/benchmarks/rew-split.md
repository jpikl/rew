# rew split

This page contains benchmarks for [rew split](../reference/rew-split.md) command.

See [benchmark setup](./setup.md) for information about testing environemt and input data files.

## Benchmark #1

Command:

```shell
hyperfine \
    --warmup 5 \
    --setup "rew loop 50 <svejk.txt >input.txt" \
    --cleanup "rm input.txt" \
    "tr ',' '\n' <input.txt" \
    "rew split ',' <input.txt"
```

Results:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `rew split ',' <input.txt` | 39.7 ± 0.9 | 37.7 | 41.8 | 1.00 |
| `tr ',' '\n' <input.txt` | 72.3 ± 1.2 | 69.4 | 75.0 | 1.82 ± 0.05 |

## Benchmark #2

Command:

```shell
hyperfine \
    --warmup 5 \
    --setup "rew loop 50 <svejk.txt >input.txt" \
    --cleanup "rm input.txt" \
    "tr ' ' '\n' <input.txt" \
    "rew split ' ' <input.txt"
```

Results:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `tr ' ' '\n' <input.txt` | 73.2 ± 4.3 | 70.4 | 95.4 | 1.00 |
| `rew split ' ' <input.txt` | 90.4 ± 1.8 | 87.5 | 96.9 | 1.23 ± 0.08 |
