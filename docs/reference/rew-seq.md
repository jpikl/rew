# rew seq

Print sequence of numbers as lines

## Usage

```
rew seq [OPTIONS] [FROM..[TO]]
```

## Arguments

<dl>
<dt><code>[FROM..[TO]]</code></dt>
<dd>
Sequence range.

Both `FROM` and `TO` are integers.

`TO` may be ommited to produce an infinite sequence.

Default value: `1..`
</dd>
</dl>

## Options

<dl>

<dt><code>-s, --step &lt;STEP&gt;</code></dt>
<dd>
Increment between numbers in sequence.

Default value: `1` (for increasing sequence), `-1` (for decreasing sequence)
</dd>

<dt><code>-h, --help</code></dt>
<dd>
Print help (see a summary with '-h')
</dd>
</dl>

## Global options

See [rew reference](rew.md#global-options) for list of additional global options.
