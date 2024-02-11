# Installation

There are multiple ways to install **rew**.

## Pre-compiled binary

Go to the [GitHub Releases page](https://github.com/jpikl/rew/releases), download the version for your platform and extract the archive.

## Build from source using Rust

Set up a [Rust development environment](https://www.rust-lang.org/tools/install), then run the following command:

```shell
cargo install rew
```

This will download **rew** sources from [crates.io](https://crates.io/crates/rew), build them and install the result into Cargo's binary directory (`~/.cargo/bin/` by default).

To uninstall it later, run the following command:

```shell
cargo uninstall rew
```

### Installing the latest development version

Run the following command:

```shell
cargo install --git https://github.com/jpikl/rew.git rew
```

This will install the most recent (and probably unstable) development version from [GitHub repository](https://github.com/jpikl/rew).
