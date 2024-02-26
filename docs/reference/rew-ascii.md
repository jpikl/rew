# rew ascii

Convert characters to ASCII

## Usage

```
rew ascii [OPTIONS]
```

## Options

<dl>

<dt><code>-d, --delete</code></dt>
<dd>

Delete non-ASCII characters instead of converting them
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

Convert input to ASCII.

```
Æneid
étude
🦀rocks!
```

```sh
rew ascii
```

```
AEneid
etude
crab rocks!
```

Delete non-ASCII characters from input.

```
Æneid
étude
🦀rocks!
```

```sh
rew ascii -d
```

```
neid
tude
rocks!
```
