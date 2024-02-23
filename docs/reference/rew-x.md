# rew x

Compose parallel shell pipelines using a pattern

## Usage

```
rew x [OPTIONS] <PATTERN>...
```

## Arguments

<dl>
<dt><code>&lt;PATTERN&gt;...</code></dt>
<dd>

Output pattern(s).

Describes how each output line is constructed from the input.

Multiple patterns are joined together using a space character.
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

<dt><code>-h, --help</code></dt>
<dd>

Print help (see a summary with '-h')
</dd>
</dl>

## Global options

See [rew reference](rew.md#global-options) for list of additional global options.

## Examples

Empty expression is replaced by input line

```sh
$ printf '%s\n' 'first' 'second' 'third' | rew x 'Hello {}!'
Hello first!
Hello second!
Hello third!
```

Expression with commands to process input line

```sh
$ printf '%s\n' 'first' 'second' 'third' | rew x 'Hello {upper | sed s/[AEIO]/_/g}!'
Hello F_RST!
Hello S_C_ND!
Hello TH_RD!
```

Multiple expressions run as parallel shell pipelines

```sh
$ printf '%s\n' 'first' 'second' 'third' | rew x '{seq}. {upper | sed s/[AEIO]/_/g}!'
1. F_RST!
2. S_C_ND!
3. TH_RD!
```
