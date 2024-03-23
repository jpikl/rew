# Benchmarks

This section contains `rew` benchmarks against various other tools, like:

- [uutils coreutils](https://github.com/uutils/coreutils)
- [GNU coreutils](https://www.gnu.org/software/coreutils/)
- [GNU bash](https://www.gnu.org/software/bash/)
- [GNU sed](https://www.gnu.org/software/sed/)
- [GNU awk](https://www.gnu.org/software/gawk/)

## Environment

All benchmarks were run using the following environment.

### System

<details>
<summary><code>neofetch --stdout os distro kernel shell model cpu memory</code></summary>

```
os: Linux 
distro: Arch Linux x86_64 
kernel: 6.7.6-201.fsync.fc39.x86_64 
shell: bash 5.2.26 
model: MS-7971 2.0 
cpu: Intel i5-6500 (4) @ 3.600GHz 
memory: 6842MiB / 32052MiB 
```

</details>
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
CPU(s) scaling MHz:                 92%
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
<details>
<summary><code>free -h</code></summary>

```
               total        used        free      shared  buff/cache   available
Mem:            31Gi       7.1Gi       1.6Gi       362Mi        23Gi        24Gi
Swap:          4.0Gi       1.6Gi       2.4Gi
```

</details>

### Software

```
hyperfine 1.18.0
rew 0.4.0
coreutils 0.0.24 (multi-call binary)
GNU Awk 5.3.0, API 4.0, PMA Avon 8-g1, (GNU MPFR 4.2.1, GNU MP 6.3.0)
GNU bash, version 5.2.26(1)-release (x86_64-pc-linux-gnu)
dd (coreutils) 9.4
head (GNU coreutils) 9.4
sed (GNU sed) 4.9
seq (GNU coreutils) 9.4
tail (GNU coreutils) 9.4
tr (GNU coreutils) 9.4
```

## Data

We are using the following files as input for benchmarking:

| | [svejk.txt](https://jpikl.github.io/data/svejk.txt)  | [rur.txt](https://jpikl.github.io/data/rur.txt) |
| ----------- | ------------------------- | ------------------- |
| Encoding    | UTF-8                     | UTF-8               |
| Characters  | Czech / German diacritics | Czech diacritics    |
| Size        | 1.31 MiB                  | 143 KiB             |
| Line count  | 5913                      | 4477                |
| Line width  | up to 4925 characters     | up to 81 characters |
| Whitespaces | Trimmed                   | Around lines        |

Both files can be downloaded using the following commands:

```shell
curl https://jpikl.github.io/data/svejk.txt -o svejk.txt
curl https://jpikl.github.io/data/rur.txt -o rur.txt
```
