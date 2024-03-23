# Benchmarks

This page contains `rew` benchmarks against various other tools:

- [GNU coreutils](https://www.gnu.org/software/coreutils/)
- [uutils coreutils](https://github.com/uutils/coreutils?tab=readme-ov-file)

## Environment

All benchmarks were run within the following environment.

System:

<details>
<summary><code>uname -a</code></summary>

```
Linux arch.jpikl-pc 6.7.6-201.fsync.fc39.x86_64 #1 SMP PREEMPT_DYNAMIC Fri Mar  1 11:09:49 UTC 2024 x86_64 GNU/Linux
```

</details>
<details>
<summary><code>lscpu</code></summary>

```
Architecture:                       x86_64
CPU op-mode(s):                     32-bit, 64-bit
Address sizes:                      39 bits physical, 48 bits virtual
Byte Order:                         Little Endian
CPU(s):                             4
On-line CPU(s) list:                0-3
Vendor ID:                          GenuineIntel
Model name:                         Intel(R) Core(TM) i5-6500 CPU @ 3.20GHz
CPU family:                         6
Model:                              94
Thread(s) per core:                 1
Core(s) per socket:                 4
Socket(s):                          1
Stepping:                           3
CPU(s) scaling MHz:                 51%
CPU max MHz:                        3600.0000
CPU min MHz:                        800.0000
BogoMIPS:                           6399.96
Flags:                              fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush dts acpi mmx fxsr sse sse2 ss ht tm pbe syscall nx pdpe1gb rdtscp lm constant_tsc art arch_perfmon pebs bts rep_good nopl xtopology nonstop_tsc cpuid aperfmperf pni pclmulqdq dtes64 monitor ds_cpl vmx smx est tm2 ssse3 sdbg fma cx16 xtpr pdcm pcid sse4_1 sse4_2 x2apic movbe popcnt tsc_deadline_timer aes xsave avx f16c rdrand lahf_lm abm 3dnowprefetch cpuid_fault pti ssbd ibrs ibpb stibp tpr_shadow flexpriority ept vpid ept_ad fsgsbase tsc_adjust bmi1 avx2 smep bmi2 erms invpcid mpx rdseed adx smap clflushopt intel_pt xsaveopt xsavec xgetbv1 xsaves dtherm ida arat pln pts hwp hwp_notify hwp_act_window hwp_epp vnmi md_clear flush_l1d arch_capabilities
Virtualization:                     VT-x
L1d cache:                          128 KiB (4 instances)
L1i cache:                          128 KiB (4 instances)
L2 cache:                           1 MiB (4 instances)
L3 cache:                           6 MiB (1 instance)
NUMA node(s):                       1
NUMA node0 CPU(s):                  0-3
Vulnerability Gather data sampling: Vulnerable: No microcode
Vulnerability Itlb multihit:        KVM: Mitigation: VMX disabled
Vulnerability L1tf:                 Mitigation; PTE Inversion; VMX conditional cache flushes, SMT disabled
Vulnerability Mds:                  Mitigation; Clear CPU buffers; SMT disabled
Vulnerability Meltdown:             Mitigation; PTI
Vulnerability Mmio stale data:      Mitigation; Clear CPU buffers; SMT disabled
Vulnerability Retbleed:             Mitigation; IBRS
Vulnerability Spec rstack overflow: Not affected
Vulnerability Spec store bypass:    Mitigation; Speculative Store Bypass disabled via prctl
Vulnerability Spectre v1:           Mitigation; usercopy/swapgs barriers and __user pointer sanitization
Vulnerability Spectre v2:           Mitigation; IBRS, IBPB conditional, STIBP disabled, RSB filling, PBRSB-eIBRS Not affected
Vulnerability Srbds:                Mitigation; Microcode
Vulnerability Tsx async abort:      Mitigation; TSX disabled
```

</details>
<details>
<summary><code>lsmem</code></summary>

```
RANGE                                  SIZE  STATE REMOVABLE  BLOCK
0x0000000000000000-0x000000008fffffff  2.3G online       yes   0-17
0x0000000100000000-0x000000086fffffff 29.8G online       yes 32-269

Memory block size:       128M
Total online memory:      32G
Total offline memory:      0B
```

</details>

Software:

<details>
<summary><code>hyperfine --version</code></summary>

```
hyperfine 1.18.0
```

</details>
<details>
<summary><code>rew --version</code></summary>

```
rew 0.4.0
```

</details>
<details>
<summary><code>coreutils --help | head -n1</code></summary>

```
coreutils 0.0.24 (multi-call binary)
```

</details>
<details>
<summary><code>seq --version</code></summary>

```
seq (GNU coreutils) 9.4
Copyright (C) 2023 Free Software Foundation, Inc.
License GPLv3+: GNU GPL version 3 or later <https://gnu.org/licenses/gpl.html>.
This is free software: you are free to change and redistribute it.
There is NO WARRANTY, to the extent permitted by law.

Written by Ulrich Drepper.
```

</details>
<details>
<summary><code>head --version</code></summary>

```
head (GNU coreutils) 9.4
Copyright (C) 2023 Free Software Foundation, Inc.
License GPLv3+: GNU GPL version 3 or later <https://gnu.org/licenses/gpl.html>.
This is free software: you are free to change and redistribute it.
There is NO WARRANTY, to the extent permitted by law.

Written by David MacKenzie and Jim Meyering.
```

</details>

## Data

The following text files were used as input data for benchmarking.

```shell
curl https://jpikl.github.io/data/svejk.txt | rew loop 100 >svejk.txt
```

- UTF-8, non-ascii (czech diacritics)
- 5913 lines (repeated 100 times)
- very long lines (up to 4952 chars)
- trimmed whitespaces

```shell
curl https://jpikl.github.io/data/rur.txt | rew loop 1000 >rur.txt
```

- UTF-8, non-ascii (czech diacritics)
- 4477 lines (repeated 1000 times)
- short lines (up to 81 chars)
- untrimmed whitespaces

## Runs

### rew seq

Command:

```shell
hyperfine \
    --warmup 5 \
    'seq 1 1 1000000' \
    'rew seq 1..1000000 1'
```

Results:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `seq 1 1 1000000` | 8.9 ± 0.5 | 8.1 | 13.0 | 1.00 |
| `rew seq 1..1000000 1` | 12.8 ± 0.4 | 12.1 | 14.2 | 1.44 ± 0.10 |

Command:

```shell
hyperfine \
    --warmup 5 \
    'seq 1 10 10000000' \
    'rew seq 1..10000000 10'
```

Results:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `rew seq 1..10000000 10` | 13.5 ± 0.3 | 12.9 | 14.6 | 1.00 |
| `seq 1 10 10000000` | 21.8 ± 0.9 | 20.6 | 25.8 | 1.61 ± 0.08 |

Command:

```shell
hyperfine \
    --warmup 5 \
    'seq 1 100 100000000' \
    'rew seq 1..100000000 100'
```

Results:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `rew seq 1..100000000 100` | 13.6 ± 0.4 | 12.8 | 16.1 | 1.00 |
| `seq 1 100 100000000` | 153.1 ± 2.7 | 150.0 | 158.7 | 11.22 ± 0.40 |
