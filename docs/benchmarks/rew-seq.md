# rew seq

This page contains benchmarks for [rew seq](../reference/rew-seq.md) command.

See [benchmark setup](./setup.md) for information about testing environemt and input data files.

## Benchmark #1

Command:

```shell
hyperfine \
    --warmup 5 \
    "seq 1 1 1000000" \
    "coreutils seq 1 1 1000000" \
    "rew seq 1..1000000 1"
```

Results:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `seq 1 1 1000000` | 10.9 ± 0.9 | 9.6 | 13.6 | 1.00 |
| `rew seq 1..1000000 1` | 13.5 ± 0.8 | 12.2 | 15.8 | 1.24 ± 0.12 |
| `coreutils seq 1 1 1000000` | 1528.1 ± 24.8 | 1499.8 | 1558.1 | 139.97 ± 11.61 |

## Benchmark #2

Command:

```shell
hyperfine \
    --warmup 5 \
    "seq 1 10 10000000" \
    "coreutils seq 1 10 10000000" \
    "rew seq 1..10000000 10"
```

Results:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `rew seq 1..10000000 10` | 13.7 ± 0.5 | 12.8 | 15.1 | 1.00 |
| `seq 1 10 10000000` | 24.3 ± 0.9 | 22.3 | 26.6 | 1.78 ± 0.10 |
| `coreutils seq 1 10 10000000` | 1488.1 ± 15.8 | 1459.6 | 1512.5 | 108.88 ± 4.52 |

## Benchmark #3

Command:

```shell
hyperfine \
    --warmup 5 \
    "seq 1 100 100000000" \
    "coreutils seq 1 100 100000000" \
    "rew seq 1..100000000 100"
```

Results:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `rew seq 1..100000000 100` | 14.0 ± 0.7 | 13.0 | 17.0 | 1.00 |
| `seq 1 100 100000000` | 160.3 ± 2.1 | 157.5 | 165.4 | 11.43 ± 0.59 |
| `coreutils seq 1 100 100000000` | 1500.0 ± 18.8 | 1469.9 | 1528.9 | 106.96 ± 5.52 |
