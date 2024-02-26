# rew split

Split input into lines using a separator

## Usage

```
rew split [OPTIONS] [SEPARATOR]
```

## Arguments

<dl>
<dt><code>[SEPARATOR]</code></dt>
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

Split input into lines on comma.

```sh
rew split ,
```

<div class="example-io">
<div class="example-io-stream">
<small><b>stdin:</b></small>
<ul>
<li><code>first,second,third</code></li>
</ul>
</div>
<div class="example-io-stream">
<small><b>stdout:</b></small>
<ul>
<li><code>first</code></li>
<li><code>second</code></li>
<li><code>third</code></li>
</ul>
</div>
</div>

Split input into lines on comma (process trailing comma).

```sh
rew split ,
```

<div class="example-io">
<div class="example-io-stream">
<small><b>stdin:</b></small>
<ul>
<li><code>first,second,third,</code></li>
</ul>
</div>
<div class="example-io-stream">
<small><b>stdout:</b></small>
<ul>
<li><code>first</code></li>
<li><code>second</code></li>
<li><code>third</code></li>
<li><code></code></li>
</ul>
</div>
</div>

Split input into lines on comma (ignore trailing comma).

```sh
rew split -t ,
```

<div class="example-io">
<div class="example-io-stream">
<small><b>stdin:</b></small>
<ul>
<li><code>first,second,third,</code></li>
</ul>
</div>
<div class="example-io-stream">
<small><b>stdout:</b></small>
<ul>
<li><code>first</code></li>
<li><code>second</code></li>
<li><code>third</code></li>
</ul>
</div>
</div>
