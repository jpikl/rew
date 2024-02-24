# rew join

Join input lines using a separator

## Usage

```
rew join [OPTIONS] [SEPARATOR]
```

## Arguments

<dl>
<dt><code>[SEPARATOR]</code></dt>
<dd>

Separator
</dd>
</dl>

## Options

<dl>

<dt><code>-t, --trailing</code></dt>
<dd>

Print trailing separator at the end
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

Join lines using comma.

```sh
$ echo 'first' > input
$ echo 'second' >> input
$ echo 'third' >> input

$ rew join , < input

first,second,third
```

Join lines using comma (include trailing comma).

```sh
$ echo 'first' > input
$ echo 'second' >> input
$ echo 'third' >> input

$ rew join -t , < input

first,second,third,
```
