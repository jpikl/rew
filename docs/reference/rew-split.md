# rew split

Split input into lines using a separator

## Usage

```
rew split [OPTIONS] <SEPARATOR>
```

## Arguments

<dl>
<dt><code>&lt;SEPARATOR&gt;</code></dt>
<dd>

Separator (single byte character)
</dd>
</dl>

## Options

<dl>

<dt><code>-t, --ignore-trailing</code></dt>
<dd>

Ignore trailing separator at the end of input
</dd>

<dt><code>-h, --help</code></dt>
<dd>

Print help (see a summary with '-h')
</dd>
</dl>

## Global options

See [rew reference](rew.md#global-options) for list of additional global options.

## Examples

Split input into lines on comma

```sh
$ echo 'first,second,third' | rew split ,
first
second
third
```

Split input into lines on comma (process trailing comma)

```sh
$ echo 'first,second,third,' | rew split ,
first
second
third

```

Split input into lines on comma (ignore trailing comma)

```sh
$ echo 'first,second,third,' | rew split -t ,
first
second
third
```
