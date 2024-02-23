# rew seq

Print sequence of numbers as lines

## Usage

```
rew seq [OPTIONS] [FROM..[TO]] [STEP]
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
<dt><code>[STEP]</code></dt>
<dd>

Increment between numbers in sequence.

Default value: `1` (for increasing sequence), `-1` (for decreasing sequence)
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

Print numbers from 1 to 3.

```sh
$ rew seq 1..3

1
2
3
```

Print numbers from 1 to 5 with step 2.

```sh
$ rew seq 1..5 2

1
3
5
```

Print numbers from 1 to -1.

```sh
$ rew seq 1..-1

1
0
-1
```

Print numbers from 1 to -3 with step -2.

```sh
$ rew seq 1..-3 -2

1
-1
-3
```
