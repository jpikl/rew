# Installation

There are multiple ways to install **rew**.

## Pre-compiled binary

Can be downloaded from [GitHub Releases page](https://github.com/jpikl/rew/releases).

## Building from sources

Set up a [Rust development environment](https://www.rust-lang.org/tools/install), then run one of the following commands:

```sh
# The latest stable version:
cargo install rew

# The latest development version:
cargo install --git https://github.com/jpikl/rew.git rew
```

The result will be stored into Cargo's binary directory.
By default, this is `~/.cargo/bin/`.
