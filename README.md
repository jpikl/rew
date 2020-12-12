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

![What rew does](docs/images/diagram.png)

Input values are assumed to be FS paths, however, rew is able to process any UTF-8 encoded text.

## Documentation

See [documentation](https://jpikl.github.io/rew) for:

   - [📦 installation][installation]
   - [🚀 usage][usage]
   - [✏️ pattern syntax][pattern]
   - [🗃 examples][examples]
   -  and more...

## License

Rew is licensed under the [MIT license](LICENSE.md).

[docs]: https://jpikl.github.io/rew
[installation]: https://jpikl.github.io/rew/installation.html
[pattern]: https://jpikl.github.io/rew/pattern.html
[usage]: https://jpikl.github.io/rew/usage.html
[examples]: https://jpikl.github.io/rew/examples.html
