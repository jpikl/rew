# rew ascii

This page contains benchmarks for [rew ascii](../reference/rew-ascii.md) command.

See [benchmark setup](./setup.md) for information about testing environemt and input data files.

## Benchmark #1

Command:

```shell
hyperfine \
    --warmup 5 \
    --setup "rew loop 2 <svejk.txt >input.txt" \
    --cleanup "rm input.txt" \
    "rew ascii <input.txt"
```

Results:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `rew ascii <input.txt` | 22.9 ± 1.0 | 21.3 | 26.3 | 1.00 |

## Benchmark #2

Command:

```shell
hyperfine \
    --warmup 5 \
    --setup "rew loop 2 <svejk.txt >input.txt" \
    --cleanup "rm input.txt" \
    "rew ascii -d <input.txt" \
    "LC_ALL=C sed 's/[^\x00-\x7F]//g' <input.txt"
```

Results:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `rew ascii -d <input.txt` | 11.7 ± 0.6 | 10.8 | 14.0 | 1.00 |
| `LC_ALL=C sed 's/[^\x00-\x7F]//g' <input.txt` | 77.2 ± 1.7 | 74.8 | 82.1 | 6.58 ± 0.38 |
