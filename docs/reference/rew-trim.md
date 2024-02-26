# rew trim

Trim whitespaces from each line.

By default, both the beginning and the end are trimmed.

## Usage

```
rew trim [OPTIONS]
```

## Options

<dl>

<dt><code>-s, --start</code></dt>
<dd>

Trim the beginning
</dd>

<dt><code>-e, --end</code></dt>
<dd>

Trim the end
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

Trim whitespaces from both sides each line.

```sh
rew trim
```

<div class="example-io">
<div class="example-io-stream">
<small><b>stdin:</b></small>
<ul>
<li><code>  spaces around  </code></li>
<li><code>  spaces before</code></li>
<li><code>spaces after   </code></li>
</ul>
</div>
<div class="example-io-stream">
<small><b>stdout:</b></small>
<ul>
<li><code>spaces around</code></li>
<li><code>spaces before</code></li>
<li><code>spaces after</code></li>
</ul>
</div>
</div>

Trim whitespaces from start of each line.

```sh
rew trim -s
```

<div class="example-io">
<div class="example-io-stream">
<small><b>stdin:</b></small>
<ul>
<li><code>  spaces around  </code></li>
<li><code>  spaces before</code></li>
<li><code>spaces after   </code></li>
</ul>
</div>
<div class="example-io-stream">
<small><b>stdout:</b></small>
<ul>
<li><code>spaces around  </code></li>
<li><code>spaces before</code></li>
<li><code>spaces after   </code></li>
</ul>
</div>
</div>

Trim whitespaces from end of each line.

```sh
rew trim -e
```

<div class="example-io">
<div class="example-io-stream">
<small><b>stdin:</b></small>
<ul>
<li><code>  spaces around  </code></li>
<li><code>  spaces before</code></li>
<li><code>spaces after   </code></li>
</ul>
</div>
<div class="example-io-stream">
<small><b>stdout:</b></small>
<ul>
<li><code>  spaces around</code></li>
<li><code>  spaces before</code></li>
<li><code>spaces after</code></li>
</ul>
</div>
</div>
