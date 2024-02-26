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

```sh
rew ascii
```

<div class="example-io">
<div class="example-io-stream">
<small><b>stdin:</b></small>
<ul>
<li><code>Æneid</code></li>
<li><code>étude</code></li>
<li><code>🦀rocks!</code></li>
</ul>
</div>
<div class="example-io-stream">
<small><b>stdout:</b></small>
<ul>
<li><code>AEneid</code></li>
<li><code>etude</code></li>
<li><code>crab rocks!</code></li>
</ul>
</div>
</div>

Delete non-ASCII characters from input.

```sh
rew ascii -d
```

<div class="example-io">
<div class="example-io-stream">
<small><b>stdin:</b></small>
<ul>
<li><code>Æneid</code></li>
<li><code>étude</code></li>
<li><code>🦀rocks!</code></li>
</ul>
</div>
<div class="example-io-stream">
<small><b>stdout:</b></small>
<ul>
<li><code>neid</code></li>
<li><code>tude</code></li>
<li><code>rocks!</code></li>
</ul>
</div>
</div>
