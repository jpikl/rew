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

```sh
rew skip 1
```

<div class="example-io">
<div class="example-io-stream">
<small><b>stdin:</b></small>
<ul>
<li><code>first</code></li>
<li><code>second</code></li>
<li><code>third</code></li>
</ul>
</div>
<div class="example-io-stream">
<small><b>stdout:</b></small>
<ul>
<li><code>second</code></li>
<li><code>third</code></li>
</ul>
</div>
</div>

Skip the first two lines.

```sh
rew skip 2
```

<div class="example-io">
<div class="example-io-stream">
<small><b>stdin:</b></small>
<ul>
<li><code>first</code></li>
<li><code>second</code></li>
<li><code>third</code></li>
</ul>
</div>
<div class="example-io-stream">
<small><b>stdout:</b></small>
<ul>
<li><code>third</code></li>
</ul>
</div>
</div>
