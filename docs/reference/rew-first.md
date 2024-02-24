# rew first

Output first N input lines

## Usage

```
rew first [OPTIONS] [COUNT]
```

## Arguments

<dl>
<dt><code>[COUNT]</code></dt>
<dd>

Number of lines to print

Default value: `1`
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

Print the first line.

```sh
$ echo 'first' > input
$ echo 'second' >> input
$ echo 'third' >> input

$ rew first < input

first
```

Print the first two lines.

```sh
$ echo 'first' > input
$ echo 'second' >> input
$ echo 'third' >> input

$ rew first 2 < input

first
second
```
