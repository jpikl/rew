# rew loop

This page contains benchmarks for [rew loop](../reference/rew-loop.md) command.

See [benchmark setup](./setup.md) for information about testing environemt and input data files.

## Benchmark #1

Command:

```shell
hyperfine \
    --warmup 5 \
    --setup "rew loop 10 <svejk.txt >input.txt" \
    --cleanup "rm input.txt" \
    "rew loop 10 <input.txt" \
    "for((i=0;i<10;i++)); do cat input.txt; done"
```

Results:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `rew loop 10 <input.txt` | 14.7 ± 0.7 | 13.6 | 17.6 | 1.00 |
| `for((i=0;i<10;i++)); do cat input.txt; done` | 44.3 ± 1.1 | 42.9 | 49.0 | 3.02 ± 0.16 |
