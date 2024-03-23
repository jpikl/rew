# rew join

This page contains benchmarks for [rew join](../reference/rew-join.md) command.

See [benchmark setup](./setup.md) for information about testing environemt and input data files.

## Benchmark #1

Command:

```shell
hyperfine \
    --warmup 5 \
    --setup "rew loop 50 <svejk.txt >input.txt" \
    --cleanup "rm input.txt" \
    "(tr '\n' ' ' && echo) <input.txt" \
    "rew join -t ' ' <input.txt"
```

Results:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `rew join -t ' ' <input.txt` | 29.8 ± 0.8 | 27.8 | 31.8 | 1.00 |
| `(tr '\n' ' ' && echo) <input.txt` | 72.2 ± 1.0 | 70.1 | 74.7 | 2.42 ± 0.07 |
