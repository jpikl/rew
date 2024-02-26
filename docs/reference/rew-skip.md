# rew skip

Skip first N input lines, output the rest

## Usage

```
rew skip [OPTIONS] [COUNT]
```

## Arguments

<dl>
<dt><code>[COUNT]</code></dt>
<dd>

Number of lines to skip
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

Skip the first line.

```
first
second
third
```

```sh
rew skip 1
```

```
second
third
```

Skip the first two lines.

```
first
second
third
```

```sh
rew skip 2
```

```
third
```
