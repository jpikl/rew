# rew x

Compose parallel shell pipelines using a pattern

## Usage

```
rew x [OPTIONS] [PATTERN]...
```

## Arguments

<dl>
<dt><code>[PATTERN]...</code></dt>
<dd>

Output pattern(s).

A pattern describes how to transform each input line into output. Multiple patterns are joined into single one using space character `' '`.

See examples (`--examples` option) for more details.
</dd>
</dl>

## Options

<dl>

<dt><code>-e, --escape &lt;CHAR&gt;</code></dt>
<dd>

Escape character for the pattern

Default value: `\`
</dd>

<dt><code>-s, --shell &lt;SHELL&gt;</code></dt>
<dd>

Shell used to evaluate `{# ...}` expressions.

Default value: `cmd` on Windows, `sh` everywhere else.

Can be also set using `SHELL` environment variable.
</dd>

<dt><code>--examples</code></dt>
<dd>

Print examples of the command usage
</dd>

<dt><code>-h, --help</code></dt>
<dd>

Print help (see a summary with '-h')
</dd>
</dl>

## Global options

See [rew reference](rew.md#global-options) for list of additional global options.

## Examples

Empty expression `{}` is replaced by input line.

```
first
second
third
```

```sh
rew x 'Hello {}'
```

```
Hello first
Hello second
Hello third
```

Expressions can call other `rew` commands to process the input.

Here, we call the `rew upper` command which converts text to uppercase.

```
first
second
third
```

```sh
rew x 'Hello {upper}'
```

```
Hello FIRST
Hello SECOND
Hello THIRD
```

Expressions can also call any external command.

Here, we remove all `aeiou` characters from text using `tr`.

```
first
second
third
```

```sh
rew x 'Hello {tr -d aeiou}'
```

```
Hello frst
Hello scnd
Hello thrd
```

Multiple commands can be joined into a pipeline.

```
first
second
third
```

```sh
rew x 'Hello {tr -d aeiou | upper}'
```

```
Hello FRST
Hello SCND
Hello THRD
```

Multiple expressions are run in parallel and their output is combined. The excution runs until one of the expressions no longer produces any output.

```
first
second
third
```

```sh
rew x '{seq}. {tr -d aeiou | upper}'
```

```
1. FRST
2. SCND
3. THRD
```

Arguments containing whitepaces must be wrapped in single `''` or double quotes `""`.

Here, we replace `aeiou` characters with space `' '`.

```
first
second
third
```

```sh
rew x 'Hello {tr aeiou " " | upper}'
```

```
Hello F RST
Hello S C ND
Hello TH RD
```

The `!` marker denotes an external command.

Here, we call the standard `seq` command instead of the built-in `rew seq`.

```
first
second
third
```

```sh
rew x '{!seq 1 3}. {}'
```

```
1. first
2. second
3. third
```

The `#` marker makes the rest of the expression to be interpreted by the current shell.

For example, the following expression is equivalent to `{sh -c 'echo a; echo b; echo c'}`

```
first
second
third
```

```sh
rew x '{# echo a; echo b; echo c}. {}'
```

```
a. first
b. second
c. third
```

A specific shell for `{# ...}` can be set using the `-s, --shell` option or the `SHELL` environment variable.

```
first
second
third
```

```sh
rew x -s bash '{# for((i=0;i<3;i++)); do echo $i; done}. {}'
```

```
0. first
1. second
2. third
```

The `:` marker is a hint that an expression does not consume stdin. Without it, the overall execution might get stuck forever due to blocked IO calls.

Only external commands need `:` to be explicitely specified. For built-in commands, `:` is detected automatically.

```sh
rew x '{seq 1..3} {: !seq 1 3} {:# echo 1; echo 2; echo 3}'
```

```
1 1 1
2 2 2
3 3 3
```

Backslash `\` can be used to escape special characters

```
first
second
third
```

```sh
rew x '\{ "{}": {seq} \}'
```

```
{ "first": 1 }
{ "second": 2 }
{ "third": 3 }
```

A custom escape character can be set using the `-e, --escape` option.

```
first
second
third
```

```sh
rew x -e% '%{ "{}": {seq} %}'
```

```
{ "first": 1 }
{ "second": 2 }
{ "third": 3 }
```

Certain special characters like `|` needs to be escaped only within a specific context.

```sh
rew x '| {echo "|"} {echo \|}'
```

```
| | |
```

Escape character can be also used to produce line feed `\n`, carriage return `\r` or tab `\t`.

```
first
second
third
```

```sh
rew x '{seq}:\n\t{}'
```

```
1:
	first
2:
	second
3:
	third
```
