# rew cat

This page contains benchmarks for [rew cat](../reference/rew-cat.md) command.

See [benchmark setup](./setup.md) for information about testing environemt and input data files.

## Benchmark #1

Command:

```shell
hyperfine \
    --warmup 5 \
    --setup "rew loop 500 <svejk.txt >input.txt" \
    --cleanup "rm input.txt" \
    "cat <input.txt" \
    "coreutils cat <input.txt" \
    "rew cat <input.txt" \
    "rew cat --bytes <input.txt" \
    "rew cat --chars <input.txt" \
    "rew cat --lines <input.txt"
```

Results:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `rew cat <input.txt` | 26.2 ± 1.4 | 23.6 | 33.2 | 1.00 |
| `coreutils cat <input.txt` | 52.2 ± 4.0 | 49.8 | 75.3 | 1.99 ± 0.18 |
| `cat <input.txt` | 89.1 ± 2.1 | 86.1 | 95.2 | 3.40 ± 0.20 |
| `rew cat --chars <input.txt` | 155.0 ± 1.7 | 152.4 | 159.1 | 5.91 ± 0.32 |
| `rew cat --bytes <input.txt` | 156.9 ± 2.3 | 152.7 | 160.8 | 5.99 ± 0.33 |
| `rew cat --lines <input.txt` | 273.6 ± 3.7 | 268.1 | 279.1 | 10.44 ± 0.57 |

## Benchmark #2

Command:

```shell
hyperfine \
    --warmup 5 \
    --setup "rew loop 5000 <rur.txt >input.txt" \
    --cleanup "rm input.txt" \
    "cat <input.txt" \
    "coreutils cat <input.txt" \
    "rew cat <input.txt" \
    "rew cat --bytes <input.txt" \
    "rew cat --chars <input.txt" \
    "rew cat --lines <input.txt"
```

Results:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `rew cat <input.txt` | 27.3 ± 1.6 | 25.1 | 38.4 | 1.00 |
| `coreutils cat <input.txt` | 54.5 ± 0.7 | 52.8 | 55.9 | 2.00 ± 0.12 |
| `cat <input.txt` | 94.2 ± 2.5 | 91.1 | 100.1 | 3.46 ± 0.22 |
| `rew cat --bytes <input.txt` | 163.6 ± 2.2 | 160.7 | 169.5 | 6.00 ± 0.36 |
| `rew cat --chars <input.txt` | 165.7 ± 2.0 | 162.7 | 170.0 | 6.08 ± 0.36 |
| `rew cat --lines <input.txt` | 753.3 ± 23.0 | 736.3 | 815.0 | 27.64 ± 1.81 |
