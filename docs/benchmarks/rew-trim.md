# rew trim

This page contains benchmarks for [rew trim](../reference/rew-trim.md) command.

See [benchmark setup](./setup.md) for information about testing environemt and input data files.

## Benchmark #1

Command:

```shell
hyperfine \
    --warmup 5 \
    --setup "rew loop 10 <svejk.txt >input.txt" \
    --cleanup "rm input.txt" \
    "rew trim <input.txt" \
    "awk '{ gsub(/^\s+|\s+$/, \"\"); print }' <input.txt" \
    "sed -E 's/^\s+|\s+$//' <input.txt"
```

Results:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `rew trim <input.txt` | 9.9 ± 0.6 | 9.1 | 12.9 | 1.00 |
| `awk '{ gsub(/^\s+\|\s+$/, ""); print }' <input.txt` | 664.7 ± 10.7 | 652.8 | 689.0 | 67.18 ± 3.97 |
| `sed -E 's/^\s+\|\s+$//' <input.txt` | 753.7 ± 11.9 | 737.6 | 771.1 | 76.18 ± 4.50 |

## Benchmark #2

Command:

```shell
hyperfine \
    --warmup 5 \
    --setup "rew loop 100 <rur.txt >input.txt" \
    --cleanup "rm input.txt" \
    "rew trim <input.txt" \
    "awk '{ gsub(/^\s+|\s+$/, \"\"); print }' <input.txt" \
    "sed -E 's/^\s+|\s+$//' <input.txt"
```

Results:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `rew trim <input.txt` | 33.7 ± 1.1 | 31.6 | 37.2 | 1.00 |
| `sed -E 's/^\s+\|\s+$//' <input.txt` | 564.1 ± 10.0 | 557.1 | 589.9 | 16.73 ± 0.61 |
| `awk '{ gsub(/^\s+\|\s+$/, ""); print }' <input.txt` | 821.2 ± 22.1 | 800.5 | 862.8 | 24.36 ± 1.02 |
