# rew upper

This page contains benchmarks for [rew upper](../reference/rew-upper.md) command.

See [benchmark setup](./setup.md) for information about testing environemt and input data files.

## Benchmark #1

Command:

```shell
hyperfine \
    --warmup 5 \
    --setup "sed s/ß/s/g <svejk.txt | rew loop 1 >input.txt" \
    --cleanup "rm input.txt" \
    "rew upper <input.txt" \
    "sed 's/\(.*\)/\U\1/' <input.txt" \
    "while IFS= read -r l;do printf '%s\n' \"${l^^}\";done <input.txt"
```

Results:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `rew upper <input.txt` | 13.3 ± 0.6 | 12.3 | 15.3 | 1.00 |
| `sed 's/\(.*\)/\U\1/' <input.txt` | 209.9 ± 3.0 | 204.6 | 214.8 | 15.75 ± 0.77 |
| `while IFS= read -r l;do printf '%s\n' "${l^^}";done <input.txt` | 242.7 ± 2.3 | 240.0 | 249.3 | 18.22 ± 0.87 |

## Benchmark #2

Command:

```shell
hyperfine \
    --warmup 5 \
    --setup "sed s/ß/s/g <rur.txt | rew loop 10 >input.txt" \
    --cleanup "rm input.txt" \
    "rew upper <input.txt" \
    "sed 's/\(.*\)/\U\1/' <input.txt" \
    "while IFS= read -r l;do printf '%s\n' \"${l^^}\";done <input.txt"
```

Results:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `rew upper <input.txt` | 11.7 ± 0.6 | 10.7 | 13.6 | 1.00 |
| `sed 's/\(.*\)/\U\1/' <input.txt` | 248.3 ± 2.3 | 245.1 | 252.6 | 21.14 ± 1.11 |
| `while IFS= read -r l;do printf '%s\n' "${l^^}";done <input.txt` | 721.6 ± 6.7 | 709.3 | 731.0 | 61.43 ± 3.22 |

## Benchmark #3

Command:

```shell
hyperfine \
    --warmup 5 \
    --setup "rew ascii <svejk.txt | rew loop 20 >input.txt" \
    --cleanup "rm input.txt" \
    "rew upper <input.txt" \
    "dd conv=ucase status=none <input.txt" \
    "coreutils dd conv=ucase status=none <input.txt" \
    "tr '[:lower:]' '[:upper:]' <input.txt" \
    "coreutils tr '[:lower:]' '[:upper:]' <input.txt" \
    "sed 's/\(.*\)/\U\1/' <input.txt"
```

Results:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `rew upper <input.txt` | 10.2 ± 0.9 | 9.3 | 18.0 | 1.00 |
| `tr '[:lower:]' '[:upper:]' <input.txt` | 27.8 ± 1.2 | 25.7 | 32.0 | 2.73 ± 0.26 |
| `dd conv=ucase status=none <input.txt` | 143.6 ± 0.9 | 142.1 | 146.0 | 14.13 ± 1.23 |
| `coreutils dd conv=ucase status=none <input.txt` | 149.7 ± 2.7 | 145.9 | 158.4 | 14.73 ± 1.30 |
| `coreutils tr '[:lower:]' '[:upper:]' <input.txt` | 821.7 ± 25.1 | 770.8 | 855.7 | 80.86 ± 7.42 |
| `sed 's/\(.*\)/\U\1/' <input.txt` | 3808.9 ± 31.0 | 3782.8 | 3873.1 | 374.80 ± 32.57 |

## Benchmark #4

Command:

```shell
hyperfine \
    --warmup 5 \
    --setup "rew ascii <rur.txt | rew loop 100 >input.txt" \
    --cleanup "rm input.txt" \
    "rew upper <input.txt" \
    "dd conv=ucase status=none <input.txt" \
    "coreutils dd conv=ucase status=none <input.txt" \
    "tr '[:lower:]' '[:upper:]' <input.txt" \
    "coreutils tr '[:lower:]' '[:upper:]' <input.txt" \
    "sed 's/\(.*\)/\U\1/' <input.txt"
```

Results:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `rew upper <input.txt` | 6.1 ± 0.4 | 5.5 | 7.8 | 1.00 |
| `tr '[:lower:]' '[:upper:]' <input.txt` | 16.4 ± 0.8 | 15.1 | 19.0 | 2.69 ± 0.22 |
| `dd conv=ucase status=none <input.txt` | 80.8 ± 1.5 | 77.7 | 84.8 | 13.25 ± 0.92 |
| `coreutils dd conv=ucase status=none <input.txt` | 84.1 ± 2.5 | 79.2 | 91.4 | 13.79 ± 1.01 |
| `coreutils tr '[:lower:]' '[:upper:]' <input.txt` | 459.0 ± 12.8 | 426.8 | 473.2 | 75.26 ± 5.46 |
| `sed 's/\(.*\)/\U\1/' <input.txt` | 2321.3 ± 10.3 | 2306.1 | 2338.5 | 380.62 ± 25.56 |
