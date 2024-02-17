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

<dt><code>-h, --help</code></dt>
<dd>
Print help (see a summary with '-h')
</dd>
</dl>

## Global options

See [rew reference](rew.md#global-options) for list of additional global options.
