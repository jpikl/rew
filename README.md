# rew

A text processing CLI tool that rewrites FS paths according to a pattern.

[![Build](https://img.shields.io/github/actions/workflow/status/jpikl/rew/ci.yml?branch=master)](https://github.com/jpikl/rew/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/codecov/c/github/jpikl/rew/master?token=9K88E1ZCBU)](https://codecov.io/gh/jpikl/rew)
[![Version](https://img.shields.io/crates/v/rew.svg)](https://crates.io/crates/rew)
[![Dependencies](https://deps.rs/repo/github/jpikl/rew/status.svg)](https://deps.rs/repo/github/jpikl/rew)
[![Downloads](https://img.shields.io/crates/d/rew)](https://crates.io/crates/rew)
[![License](https://img.shields.io/crates/l/rew.svg)](https://github.com/jpikl/rew/blob/master/LICENSE.md)

## How rew works

1. Reads values from standard [input](https://jpikl.github.io/rew/input.html).
2. Rewrites them according to a [pattern](https://jpikl.github.io/rew/pattern.html).
3. Prints results to standard [output](https://jpikl.github.io/rew/output.html).

![How rew works](docs/images/diagram.svg)

Input values are assumed to be FS paths, however, `rew` is able to process any UTF-8 encoded text.

```bash
find -iname '*.jpeg' | rew 'img_{C}.{e|l|r:e}'
```

`rew` is also distributed with two accompanying utilities (`mvb` and `cpb`) which move/copy files and directories, based on `rew` output.

```bash
find -iname '*.jpeg' | rew 'img_{C}.{e|l|r:e}' -d | mvb
```

## Documentation

- [📦 Installation](https://jpikl.github.io/rew/install)
- [🚀 Usage](https://jpikl.github.io/rew/usage)
- [✏️ Pattern](https://jpikl.github.io/rew/pattern)
  - [🛤 Path filters](https://jpikl.github.io/rew/filters/path)
  - [🆎 Substring filters](https://jpikl.github.io/rew/filters/substr)
  - [📊 Field filters](https://jpikl.github.io/rew/filters/field)
  - [🔍 Replace filters](https://jpikl.github.io/rew/filters/replace)
  - [⭐️ Regex filters](https://jpikl.github.io/rew/filters/regex)
  - [🎨 Format filters](https://jpikl.github.io/rew/filters/format)
  - [🏭 Generators](https://jpikl.github.io/rew/filters/generate)
- [⌨️ Input](https://jpikl.github.io/rew/input)
- [💬 Output](https://jpikl.github.io/rew/output)
- [🔬 Comparison](https://jpikl.github.io/rew/comparison)
- [🗃 Examples](https://jpikl.github.io/rew/examples)
- [📈 Changelog](https://jpikl.github.io/rew/changelog)

## License

`rew` is licensed under the [MIT license](LICENSE.md).
