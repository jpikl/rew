# rew x

This page contains benchmarks for [rew x](../reference/rew-x.md) command.

See [benchmark setup](./setup.md) for information about testing environemt and input data files.

## Benchmark #1

Command:

```shell
hyperfine \
    --warmup 5 \
    --setup "rew loop 100 <svejk.txt >input.txt" \
    --cleanup "rm input.txt" \
    "rew x '{}' <input.txt" \
    "rew x '{cat}' <input.txt" \
    "rew x '{cat | cat}' <input.txt" \
    "rew x '{cat | cat | cat}' <input.txt" \
    "rew x '{cat | cat | cat | cat}' <input.txt" \
    "rew x '{cat | cat | cat | cat | cat}' <input.txt"
```

Results:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `rew x '{}' <input.txt` | 58.6 ± 3.1 | 54.7 | 70.3 | 1.00 |
| `rew x '{cat \| cat \| cat}' <input.txt` | 115.7 ± 7.1 | 106.6 | 132.1 | 1.98 ± 0.16 |
| `rew x '{cat \| cat}' <input.txt` | 132.1 ± 37.8 | 95.0 | 214.8 | 2.26 ± 0.66 |
| `rew x '{cat \| cat \| cat \| cat \| cat}' <input.txt` | 144.6 ± 12.7 | 123.7 | 163.6 | 2.47 ± 0.25 |
| `rew x '{cat \| cat \| cat \| cat}' <input.txt` | 146.3 ± 26.7 | 112.8 | 219.7 | 2.50 ± 0.48 |
| `rew x '{cat}' <input.txt` | 188.8 ± 63.9 | 110.4 | 288.9 | 3.22 ± 1.10 |

## Benchmark #2

Command:

```shell
hyperfine \
    --warmup 5 \
    --setup "rew loop 1000 <rur.txt >input.txt" \
    --cleanup "rm input.txt" \
    "rew x '{}' <input.txt" \
    "rew x '{cat}' <input.txt" \
    "rew x '{cat | cat}' <input.txt" \
    "rew x '{cat | cat | cat}' <input.txt" \
    "rew x '{cat | cat | cat | cat}' <input.txt" \
    "rew x '{cat | cat | cat | cat | cat}' <input.txt"
```

Results:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `rew x '{}' <input.txt` | 169.2 ± 29.1 | 153.6 | 246.1 | 1.00 |
| `rew x '{cat \| cat}' <input.txt` | 215.8 ± 9.7 | 199.3 | 234.3 | 1.28 ± 0.23 |
| `rew x '{cat \| cat \| cat}' <input.txt` | 235.4 ± 38.7 | 204.0 | 326.8 | 1.39 ± 0.33 |
| `rew x '{cat \| cat \| cat \| cat \| cat}' <input.txt` | 251.2 ± 25.7 | 227.2 | 321.9 | 1.48 ± 0.30 |
| `rew x '{cat \| cat \| cat \| cat}' <input.txt` | 252.7 ± 42.8 | 215.2 | 339.4 | 1.49 ± 0.36 |
| `rew x '{cat}' <input.txt` | 307.6 ± 61.9 | 207.3 | 399.0 | 1.82 ± 0.48 |

## Benchmark #3
> **Note:** Each benchmarked command produces different output.

Command:

```shell
hyperfine \
    --warmup 5 \
    --setup "rew loop 100 <svejk.txt >input.txt" \
    --cleanup "rm input.txt" \
    "rew x '{}' <input.txt" \
    "rew x '{cat}' <input.txt" \
    "rew x '{cat}{cat}' <input.txt" \
    "rew x '{cat}{cat}{cat}' <input.txt" \
    "rew x '{cat}{cat}{cat}{cat}' <input.txt" \
    "rew x '{cat}{cat}{cat}{cat}{cat}' <input.txt"
```

Results:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `rew x '{}' <input.txt` | 60.0 ± 1.9 | 55.8 | 63.9 | 1.00 |
| `rew x '{cat}' <input.txt` | 210.5 ± 58.3 | 97.8 | 319.8 | 3.51 ± 0.98 |
| `rew x '{cat}{cat}' <input.txt` | 252.6 ± 68.1 | 162.0 | 393.5 | 4.21 ± 1.14 |
| `rew x '{cat}{cat}{cat}' <input.txt` | 345.9 ± 68.5 | 269.7 | 485.4 | 5.77 ± 1.16 |
| `rew x '{cat}{cat}{cat}{cat}' <input.txt` | 469.9 ± 92.3 | 348.7 | 650.0 | 7.83 ± 1.56 |
| `rew x '{cat}{cat}{cat}{cat}{cat}' <input.txt` | 517.3 ± 67.2 | 452.0 | 686.5 | 8.62 ± 1.15 |

## Benchmark #4
> **Note:** Each benchmarked command produces different output.

Command:

```shell
hyperfine \
    --warmup 5 \
    --setup "rew loop 1000 <rur.txt >input.txt" \
    --cleanup "rm input.txt" \
    "rew x '{}' <input.txt" \
    "rew x '{cat}' <input.txt" \
    "rew x '{cat}{cat}' <input.txt" \
    "rew x '{cat}{cat}{cat}' <input.txt" \
    "rew x '{cat}{cat}{cat}{cat}' <input.txt" \
    "rew x '{cat}{cat}{cat}{cat}{cat}' <input.txt"
```

Results:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `rew x '{}' <input.txt` | 158.6 ± 3.8 | 154.8 | 166.0 | 1.00 |
| `rew x '{cat}' <input.txt` | 332.9 ± 80.0 | 207.0 | 467.4 | 2.10 ± 0.51 |
| `rew x '{cat}{cat}' <input.txt` | 388.9 ± 85.4 | 309.6 | 571.8 | 2.45 ± 0.54 |
| `rew x '{cat}{cat}{cat}' <input.txt` | 508.2 ± 85.9 | 452.7 | 724.8 | 3.20 ± 0.55 |
| `rew x '{cat}{cat}{cat}{cat}' <input.txt` | 714.5 ± 44.8 | 667.3 | 797.1 | 4.51 ± 0.30 |
| `rew x '{cat}{cat}{cat}{cat}{cat}' <input.txt` | 809.0 ± 104.2 | 704.4 | 969.2 | 5.10 ± 0.67 |
