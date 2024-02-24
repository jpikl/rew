# rew loop

Repeatedly output all captured input

## Usage

```
rew loop [OPTIONS] [COUNT]
```

## Arguments

<dl>
<dt><code>[COUNT]</code></dt>
<dd>

How many times do the repetition (default: forever)
</dd>
</dl>

## Options

<dl>

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

Repeat all input two times.

```sh
$ echo 'first' > input
$ echo 'second' >> input

$ rew loop 2 < input

first
second
first
second
```
