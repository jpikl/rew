# 🗃 Examples

> ℹ️ Use `rew --explain <pattern>` to print detailed explanation what a certain pattern does.

Print contents of your current working directory as absolute paths.

```bash
rew '{a}' *    # Paths are passed as arguments, wildcard expansion is done by shell
ls | rew '{a}' # Paths are read from standard input
```

Rename all `*.jpeg` files to `*.jpg`.

```bash
find -name '*.jpeg' | rew -b '{B}.jpg' | mvb -v
```

Same thing but we use `rew` to generate executable shell code.

```bash
find -name '*.jpeg' | rew -q 'mv -v {} {B}.jpg' | sh
```

Make backup copy of each `*.txt` file with `.txt.bak` extension in the same directory.

```bash
find -name '*.txt'  | rew -b '{}.bak'  | cpb -v
```

Copy `*.txt` files (keep directory structure) to the `~/Backup` directory.

```bash
find -name '*.txt'  | rew -b "$HOME/Backup/{p}"  | cpb -v
```

Copy `*.txt` files (flatten directory structure) to the `~/Backup` directory.

```bash
find -name '*.txt'  | rew -b "$HOME/Backup/{f}"  | cpb -v
```

Same thing but we append randomly generated suffix after base name to avoid name collisions.

```bash
find -name '*.txt'  | rew -b "$HOME/Backup/{b}_{U}.{e}"  | cpb -v
```

Flatten directory structure `./dir/subdir/` to `./dir_subdir/`.

```bash
find -mindepth 2 -maxdepth 2 -type d | rew -b '{D}_{F}' | mvb -v
```

Normalize base names of files to `file_001`, `file_002`, ...

```bash
find -type f | rew -b '{d}/file_{C|<3:0}{E}' | mvb -v
```

Print the first word of each line with removed diacritics (accents).

```bash
rew '{=\S+|i}' <input.txt
```

Swap the first and second column in a CSV file.

```bash
rew -e'([^:]*):([^:]*):(.*)' '{$2}:{$1}:{$3}' <input.csv >output.csv
```

Same thing but we use regex replace filter.

```bash
rew '{s/([^:]*):([^:]*):(.*)/$2:$1:$3}' <input.csv >output.csv
```

Print `PATH` variable entries as lines.

````bash
echo "$PATH" | rew -d: # PATH entries are delimited by ':'
````

Replace tabs with 4 spaces in a file.

````bash
rew -rR '{R:%t:    }' <input.txt >output.txt # Read/write file content as a whole
````

Normalize line endings in a file to `LF`.

````bash
rew <input.txt >output.txt # LF is the default output delimiter
````

Normalize line endings in a file to `CR+LF`.

````bash
rew -D$'\r\n'   <input.txt >output.txt # CR+LF delimiter using -D option
rew -R '{}%r%n' <input.txt >output.txt # CR+LF delimiter in pattern
````
