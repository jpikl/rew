# rew skip

Skip first N input lines, output the rest

## Usage

```
rew skip [OPTIONS] <COUNT>
```

## Arguments

<dl>
<dt><code>&lt;COUNT&gt;</code></dt>
<dd>

Number of lines to skip
</dd>
</dl>

## Options

<dl>

<dt><code>-h, --help</code></dt>
<dd>

Print help (see a summary with '-h')
</dd>
</dl>

## Global options

See [rew reference](rew.md#global-options) for list of additional global options.

## Examples

Skip the first line

```sh
$ printf '%s\n' 'first' 'second' 'third' | rew skip 1
second
third
```

Skip the first two lines

```sh
$ printf '%s\n' 'first' 'second' 'third' | rew skip 2
third
```
