# Rew

Rew is a text processing CLI tool that rewrites FS paths according to a pattern.

[![Version](https://img.shields.io/crates/v/rew.svg)](https://crates.io/crates/rew)
[![Dependencies](https://deps.rs/repo/github/jpikl/rew/status.svg)](https://deps.rs/repo/github/jpikl/rew)
[![Downloads](https://img.shields.io/crates/d/rew)](https://crates.io/crates/rew)
[![License](https://img.shields.io/crates/l/rew.svg)](https://github.com/jpikl/rew/blob/master/LICENSE.md)
<br>
[![Build status](https://img.shields.io/github/workflow/status/jpikl/rew/Build/master?event=push&label=build%20%28master%29&logo=github)](https://github.com/jpikl/rew/actions?query=workflow%3ABuild+branch%3Amaster)
[![Code coverage](https://img.shields.io/codecov/c/github/jpikl/rew/master?label=coverage%20%28master%29&logo=codecov&token=9K88E1ZCBU)](https://codecov.io/gh/jpikl/rew/branch/master)
<br>
[![Build status](https://img.shields.io/github/workflow/status/jpikl/rew/Build/develop?event=push&label=build%20%28develop%29&logo=github)](https://github.com/jpikl/rew/actions?query=workflow%3ABuild+branch%3Adevelop)
[![Code coverage](https://img.shields.io/codecov/c/github/jpikl/rew/develop?label=coverage%20%28develop%29&logo=codecov&token=9K88E1ZCBU)](https://codecov.io/gh/jpikl/rew/branch/develop)

## What rew does

1. Reads values from standard input.
2. Rewrites them according to a pattern.
3. Prints results to standard output.

![What rew does](docs/images/diagram.svg)

Input values are assumed to be FS paths, however, rew is able to process any UTF-8 encoded text.

## Documentation

- [📦 Installation](https://jpikl.github.io/rew/installation.html)
- [🚀 Usage](https://jpikl.github.io/rew/usage.html)
- [✏️ Pattern](https://jpikl.github.io/rew/pattern.html)
  - [🛤 Path filters](https://jpikl.github.io/rew/filters/path.html)
  - [🆎 Substring filters](https://jpikl.github.io/rew/filters/substr.html)
  - [🔍 Replace filters](https://jpikl.github.io/rew/filters/replace.html)
  - [⭐️ Regex filters](https://jpikl.github.io/rew/filters/regex.html)
  - [🎨 Format filters](https://jpikl.github.io/rew/filters/format.html)
  - [🏭 Generators](https://jpikl.github.io/rew/filters/generators.html)
- [⌨️ Input](https://jpikl.github.io/rew/input.html)
- [💬 Output](https://jpikl.github.io/rew/output.html)
- [🔬 Comparison with similar tools](https://jpikl.github.io/rew/comparison.html)
- [🗃 Examples](https://jpikl.github.io/rew/examples.html)

## License

Rew is licensed under the [MIT license](LICENSE.md).
