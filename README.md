# Rew

Rew is a text processing CLI tool that rewrites FS paths according to a pattern.

<sup>master:</sup>
[![Build status](https://github.com/jpikl/rew/workflows/Build/badge.svg?branch=master&event=push)](https://github.com/jpikl/rew/actions?query=workflow%3ABuild+branch%3Amaster)
[![Code coverage](https://codecov.io/gh/jpikl/rew/branch/master/graph/badge.svg?token=9K88E1ZCBU)](https://codecov.io/gh/jpikl/rew/branch/master)
[![crates.io](https://img.shields.io/crates/v/rew.svg)](https://crates.io/crates/rew)
<br>
<sup>develop:</sup>
[![Build status](https://github.com/jpikl/rew/workflows/Build/badge.svg?branch=develop&event=push)](https://github.com/jpikl/rew/actions?query=workflow%3ABuild+branch%3Adevelop)
[![Code coverage](https://codecov.io/gh/jpikl/rew/branch/develop/graph/badge.svg?token=9K88E1ZCBU)](https://codecov.io/gh/jpikl/rew/branch/develop)

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
