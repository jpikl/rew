# rew skip

This page contains benchmarks for [rew skip](../reference/rew-skip.md) command.

See [benchmark setup](./setup.md) for information about testing environemt and input data files.

## Benchmark #1

Command:

```shell
hyperfine \
    --warmup 5 \
    --setup "rew loop 100 <svejk.txt >input.txt" \
    --cleanup "rm input.txt" \
    "tail -n +200001 <input.txt" \
    "coreutils tail -n +200001 <input.txt" \
    "rew skip 200000 <input.txt"
```

Results:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `rew skip 200000 <input.txt` | 18.1 ± 0.8 | 16.6 | 20.8 | 1.00 |
| `coreutils tail -n +200001 <input.txt` | 26.5 ± 3.4 | 24.2 | 57.9 | 1.47 ± 0.20 |
| `tail -n +200001 <input.txt` | 74.2 ± 1.8 | 71.7 | 78.6 | 4.11 ± 0.21 |

## Benchmark #2

Command:

```shell
hyperfine \
    --warmup 5 \
    --setup "rew loop 1000 <rur.txt >input.txt" \
    --cleanup "rm input.txt" \
    "tail -n +2000001 <input.txt" \
    "coreutils tail -n +2000001 <input.txt" \
    "rew skip 2000000 <input.txt"
```

Results:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `rew skip 2000000 <input.txt` | 41.9 ± 1.2 | 39.5 | 45.5 | 1.00 |
| `coreutils tail -n +2000001 <input.txt` | 58.1 ± 2.2 | 55.4 | 66.3 | 1.39 ± 0.07 |
| `tail -n +2000001 <input.txt` | 90.6 ± 1.9 | 87.5 | 94.9 | 2.16 ± 0.08 |
