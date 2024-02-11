# rew

[![Build](https://img.shields.io/github/actions/workflow/status/jpikl/rew/ci.yml?branch=master)](https://github.com/jpikl/rew/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/codecov/c/github/jpikl/rew/master?token=9K88E1ZCBU)](https://codecov.io/gh/jpikl/rew)
[![Version](https://img.shields.io/crates/v/rew.svg)](https://crates.io/crates/rew)
[![Dependencies](https://deps.rs/repo/github/jpikl/rew/status.svg)](https://deps.rs/repo/github/jpikl/rew)
[![Downloads](https://img.shields.io/crates/d/rew)](https://crates.io/crates/rew)
[![License](https://img.shields.io/crates/l/rew.svg)](https://github.com/jpikl/rew/blob/master/LICENSE.md)

<!-- ANCHOR: description -->

> _The Swiss Army Knife of line-oriented text processing._

**rew** provides you with various tools for:

- Text filtering, transformation and generation.
- File system paths manipulation.
- Parallel shell pipeline composition.
- Shell metaprogramming (code generation).

All tools are distributed in a single binary (in [BusyBox](https://www.busybox.net) style).

<!-- ANCHOR_END: description -->

## Documentation

Visit [rew website](https://jpikl.github.io/rew) for installation, usage, examples and more.

<!-- ANCHOR: showcase -->

## Showcase

Let's start with output of the standard Unix `find` command:

```sh
> find src -type f

src/README.TXT
src/image_1.JPG
src/image_2.JPEG
```

Use `rew` subcommands to query components of each path:

```sh
> find src -type f | rew base

README
image_1
image_2

> find src -type f | rew ext

TXT
JPG
JPEG
```

Combine multiple `rew` subcommands to get normalized results:

```sh
> find src -type f | rew ext | rew lower

txt
jpg
jpeg

> find src -type f | rew ext | rew lower | rew replace eg g

txt
jpg
jpg
```

Compose multiple pipelines using `x` subcommand:

```sh
> find src -type f | rew x 'dst/{base}.{ext | lower | replace eg g}'

dst/README.txt
dst/image_1.jpg
dst/image_2.jpg
```

Update the pattern to generate shell code:

```sh
> find src -type f | rew x 'mv {} dst/{base}.{ext | lower | replace eg g}'

mv src/README.TXT dst/README.txt
mv src/image_1.JPG dst/image_1.jpg
mv src/image_2.JPEG dst/image_2.jpg
```

And pipe it into a shell for execution:

```sh
> find src -type f | rew x 'mv {} dst/{base}.{ext | lower | replace eg g}' | sh
```

Or into a tool like [GNU parallel](https://www.gnu.org/software/parallel/parallel.html) for even faster execution:

```sh
> find src -type f | rew x 'mv {} dst/{base}.{ext | lower | replace eg g}' | parallel
```

You are not limited only to `rew` subcommands. Call whatever tool you like.
For example, let's use `sed` instead of `rew replace`:

```sh
> find src -type f | rew x 'mv {} dst/{base}.{ext | lower | sed s/eg/g/}' | sh
```

Have fun using rew!

<!-- ANCHOR_END: showcase -->

## License

**rew** source and documentation are released under the [MIT License](LICENSE.md).
