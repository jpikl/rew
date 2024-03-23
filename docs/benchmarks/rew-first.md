# rew first

This page contains benchmarks for [rew first](../reference/rew-first.md) command.

See [benchmark setup](./setup.md) for information about testing environemt and input data files.

## Benchmark #1

Command:

```shell
hyperfine \
    --warmup 5 \
    --setup "rew loop 100 <svejk.txt >input.txt" \
    --cleanup "rm input.txt" \
    "head -n 200000 <input.txt" \
    "coreutils head -n 200000 <input.txt" \
    "rew first 200000 <input.txt"
```

Results:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `rew first 200000 <input.txt` | 18.9 ± 1.1 | 17.2 | 25.7 | 1.00 |
| `coreutils head -n 200000 <input.txt` | 25.0 ± 0.8 | 23.2 | 28.4 | 1.32 ± 0.09 |
| `head -n 200000 <input.txt` | 48.6 ± 0.9 | 46.6 | 50.6 | 2.57 ± 0.15 |

## Benchmark #2

Command:

```shell
hyperfine \
    --warmup 5 \
    --setup "rew loop 1000 <rur.txt >input.txt" \
    --cleanup "rm input.txt" \
    "head -n 2000000 <input.txt" \
    "coreutils head -n 2000000 <input.txt" \
    "rew first 2000000 <input.txt"
```

Results:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `rew first 2000000 <input.txt` | 45.4 ± 1.0 | 43.1 | 48.6 | 1.00 |
| `coreutils head -n 2000000 <input.txt` | 51.7 ± 1.4 | 48.5 | 56.4 | 1.14 ± 0.04 |
| `head -n 2000000 <input.txt` | 85.1 ± 1.3 | 82.9 | 89.2 | 1.87 ± 0.05 |
