# rew last

This page contains benchmarks for [rew last](../reference/rew-last.md) command.

See [benchmark setup](./setup.md) for information about testing environemt and input data files.

## Benchmark #1

Command:

```shell
hyperfine \
    --warmup 5 \
    --setup "rew loop 100 <svejk.txt >input.txt" \
    --cleanup "rm input.txt" \
    "tail -n 200000 <input.txt" \
    "coreutils tail -n 200000 <input.txt" \
    "rew last 200000 <input.txt"
```

Results:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `tail -n 200000 <input.txt` | 54.4 ± 0.8 | 52.7 | 56.1 | 1.00 |
| `coreutils tail -n 200000 <input.txt` | 63.5 ± 0.8 | 61.2 | 66.4 | 1.17 ± 0.02 |
| `rew last 200000 <input.txt` | 84.4 ± 1.8 | 82.3 | 92.6 | 1.55 ± 0.04 |

## Benchmark #2

Command:

```shell
hyperfine \
    --warmup 5 \
    --setup "rew loop 1000 <rur.txt >input.txt" \
    --cleanup "rm input.txt" \
    "tail -n 2000000 <input.txt" \
    "coreutils tail -n 2000000 <input.txt" \
    "rew last 2000000 <input.txt"
```

Results:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `tail -n 2000000 <input.txt` | 92.6 ± 1.0 | 90.9 | 94.9 | 1.00 |
| `coreutils tail -n 2000000 <input.txt` | 100.2 ± 1.6 | 98.1 | 104.4 | 1.08 ± 0.02 |
| `rew last 2000000 <input.txt` | 144.4 ± 2.8 | 141.8 | 150.2 | 1.56 ± 0.03 |
