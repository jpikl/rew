# 🔬 Comparison with similar tools

## rew vs rename/prename

- Unlike `rename`, `rew` can read input paths directly from standard input.
  Use of `xargs` to pass output of `find` or [`fd`][fd] is not needed.
- Unlike `rename`, `rew` is only a text-processing tool and it is unable to rename files.
  You have to use accompanying `mvb`/`cpb` utilities or you can generate executable shell code.

```bash
find -name '*.jpeg' | xargs rename .jpeg .jpg     # Rename *.jpeg files to *.jpg
find -name '*.jpeg' | rew -d '{B}.jpg' | mvb      # Same thing using rew + mvb
find -name '*.jpeg' | rew -q 'mv {} {B}.jpg' | sh # Same thing using rew + mv + sh
```

## rew vs coreutils

Like `pwd`, `rew` is able to print your current working directory.

```bash
pwd          # Print your current working directory
rew '{w}' '' # Same thing using rew
```

Like `basename`, `rew` is able to strip directory and suffix from a path.

```bash
basename 'dir/file.txt' '.txt' # Print base name without the ".txt" extension
rew '{b}' 'dir/file.txt'       # Same thing using rew, no need to specify an extension
```

Like `dirname`, `rew` is able to strip last component from a path.

```bash
dirname 'dir/file.txt'   # Print directory name
rew '{D}' 'dir/file.txt' # Same thing using rew
```

Like `realpath`, `rew` is able to resolve a path.

```bash
realpath -e '/usr/../home'            # Print canonical path
rew '{P}' '/usr/../home'              # Same thing using rew
realpath --relative-to='/home' '/usr' # Print path relative to a directory
rew -w '/home' '{A}' '/usr'           # Same thing using rew
```

## rew vs grep

Like `grep`, `rew` is able to print match of a regular expression.

```bash
echo "123 abc 456" | grep -Po '\d+' # Extract all numbers from a string
echo "123 abc 456" | rew '{=\d+}'   # Same thing using rew (but only the first number)
```

## rew vs sed/sd

Like `sed` or [`sd`][sd], `rew` is able to replace text using a regular expression.

```bash
echo "123 abc 456" | sed -E 's/([0-9]+)/_\1_/g' # Put underscores around numbers
echo "123 abc 456" | sd '(\d+)' '_${1}_'        # Same thing using sd
echo "123 abc 456" | rew '{S:(\d+):_$1_}'       # Same thing using rew
```

## rew vs awk

`awk` is obviously more powerful tool than `rew`.
However, there are some use cases where `rew` can replace `awk` using more compact pattern syntax.

```bash
awk '{print substr($0,2,3)}' # Substring from index 2 with length 3
rew '{#2+3}'                 # Same thing using rew
```

[fd]: https://github.com/sharkdp/fd
[sd]: https://github.com/chmln/sd
