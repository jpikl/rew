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

<dt><code>-h, --help</code></dt>
<dd>

Print help (see a summary with '-h')
</dd>
</dl>

## Global options

See [rew reference](rew.md#global-options) for list of additional global options.

## Examples

Convert input to ASCII

```sh
> printf '%s\n' 'Æneid' 'étude' '🦀rocks!' | rew ascii
AEneid
etude
crab rocks!
```

Delete non-ASCII characters from input

```sh
> printf '%s\n' 'Æneid' 'étude' '🦀rocks!' | rew ascii -d
neid
tude
rocks!
```
