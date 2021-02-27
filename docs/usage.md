# 🚀 Usage

```bash
rew [options] [--] [pattern] [values]...
````

When no values are provided, they are read from standard input instead.

```bash
input | rew [options] [--] [pattern]
```

When no pattern is provided, values are directly copied to standard output.

```bash
input | rew [options]
```

Use `-b, --diff` flag when piping output to `mvb`/`cpb` utilities to perform bulk move/copy.

```bash
rew [options] [--] [pattern] -b | mvb
```

Use `-h` flag to print short help, `--help` to print detailed help.
