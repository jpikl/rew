# 🚀 Usage

By default, input values are read as lines from standard input.

```bash
some_other_command | rew [options] [pattern]
```

Input values can be also passed as additional arguments.

```bash
rew [options] [pattern] [--] <value>...
```

Use `-b, --diff` flag when piping output to `mvb`/`cpb` utilities to perform bulk move/copy.

```bash
rew [options] [pattern] -b | mvb
```

Use `-h` flag to print short help, `--help` to print detailed help.
