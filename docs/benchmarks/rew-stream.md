# rew stream

This page contains benchmarks for [rew stream](../reference/rew-stream.md) command.

See [benchmark setup](./setup.md) for information about testing environemt and input data files.

## Benchmark #1

Command:

```shell
hyperfine \
    --warmup 5 \
    --setup "seq 1 100000 >input.txt" \
    --cleanup "rm input.txt" \
    "rew stream {1..100000}" \
    "printf '%s\n' {1..100000}" \
    "xargs rew stream <input.txt" \
    "xargs printf '%s\n' <input.txt"
```

Results:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `xargs printf '%s\n' <input.txt` | 90.2 ± 1.5 | 86.7 | 92.3 | 1.00 |
| `xargs rew stream <input.txt` | 133.5 ± 2.0 | 130.3 | 137.1 | 1.48 ± 0.03 |
| `rew stream {1..100000}` | 199.3 ± 2.1 | 195.1 | 203.5 | 2.21 ± 0.04 |
| `printf '%s\n' {1..100000}` | 207.1 ± 2.1 | 204.4 | 210.6 | 2.30 ± 0.04 |
