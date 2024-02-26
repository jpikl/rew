# rew lower

Convert characters to lowercase

## Usage

```
rew lower [OPTIONS]
```

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

Convert characters to lowercase.

```sh
rew lower
```

<div class="example-io">
<div class="example-io-stream">
<small><b>stdin:</b></small>
<ul>
<li><code>hello world</code></li>
<li><code>Hello World</code></li>
<li><code>HELLO WORLD</code></li>
</ul>
</div>
<div class="example-io-stream">
<small><b>stdout:</b></small>
<ul>
<li><code>hello world</code></li>
<li><code>hello world</code></li>
<li><code>hello world</code></li>
</ul>
</div>
</div>
