# rew lower

This page contains benchmarks for [rew lower](../reference/rew-lower.md) command.

See [benchmark setup](./setup.md) for information about testing environemt and input data files.

## Benchmark #1

Command:

```shell
hyperfine \
    --warmup 5 \
    --setup "rew loop 1 <svejk.txt >input.txt" \
    --cleanup "rm input.txt" \
    "rew lower <input.txt" \
    "sed 's/\(.*\)/\L\1/' <input.txt" \
    "while IFS= read -r l;do printf '%s\n' \"${l,,}\";done <input.txt"
```

Results:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `rew lower <input.txt` | 13.5 ± 0.6 | 12.6 | 14.9 | 1.00 |
| `sed 's/\(.*\)/\L\1/' <input.txt` | 209.3 ± 2.1 | 206.7 | 214.3 | 15.53 ± 0.67 |
| `while IFS= read -r l;do printf '%s\n' "${l,,}";done <input.txt` | 240.0 ± 3.6 | 234.6 | 246.7 | 17.80 ± 0.80 |

## Benchmark #2

Command:

```shell
hyperfine \
    --warmup 5 \
    --setup "rew loop 10 <rur.txt >input.txt" \
    --cleanup "rm input.txt" \
    "rew lower <input.txt" \
    "sed 's/\(.*\)/\L\1/' <input.txt" \
    "while IFS= read -r l;do printf '%s\n' \"${l,,}\";done <input.txt"
```

Results:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `rew lower <input.txt` | 11.9 ± 0.6 | 11.1 | 13.8 | 1.00 |
| `sed 's/\(.*\)/\L\1/' <input.txt` | 250.3 ± 2.6 | 246.6 | 254.5 | 21.06 ± 1.01 |
| `while IFS= read -r l;do printf '%s\n' "${l,,}";done <input.txt` | 715.4 ± 9.2 | 707.1 | 733.5 | 60.19 ± 2.92 |

## Benchmark #3

Command:

```shell
hyperfine \
    --warmup 5 \
    --setup "rew ascii <svejk.txt | rew loop 20 >input.txt" \
    --cleanup "rm input.txt" \
    "rew lower <input.txt" \
    "dd conv=lcase status=none <input.txt" \
    "tr '[:upper:]' '[:lower:]' <input.txt" \
    "sed 's/\(.*\)/\L\1/' <input.txt"
```

Results:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `rew lower <input.txt` | 10.4 ± 1.9 | 9.3 | 27.5 | 1.00 |
| `tr '[:upper:]' '[:lower:]' <input.txt` | 27.7 ± 0.9 | 25.9 | 30.0 | 2.67 ± 0.50 |
| `dd conv=lcase status=none <input.txt` | 143.4 ± 1.4 | 141.0 | 145.5 | 13.86 ± 2.58 |
| `sed 's/\(.*\)/\L\1/' <input.txt` | 3874.5 ± 69.6 | 3821.6 | 4032.9 | 374.25 ± 69.94 |

## Benchmark #4

Command:

```shell
hyperfine \
    --warmup 5 \
    --setup "rew ascii <rur.txt | rew loop 100 >input.txt" \
    --cleanup "rm input.txt" \
    "rew lower <input.txt" \
    "dd conv=lcase status=none <input.txt" \
    "tr '[:upper:]' '[:lower:]' <input.txt" \
    "sed 's/\(.*\)/\L\1/' <input.txt"
```

Results:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `rew lower <input.txt` | 6.7 ± 0.7 | 5.7 | 9.6 | 1.00 |
| `tr '[:upper:]' '[:lower:]' <input.txt` | 16.7 ± 1.0 | 15.2 | 21.6 | 2.50 ± 0.31 |
| `dd conv=lcase status=none <input.txt` | 81.3 ± 1.7 | 79.3 | 88.4 | 12.19 ± 1.34 |
| `sed 's/\(.*\)/\L\1/' <input.txt` | 2398.9 ± 37.3 | 2353.8 | 2493.8 | 359.73 ± 39.21 |
