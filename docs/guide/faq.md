# FAQ

## What does the "rew" mean?

**rew** is a short for _"rewrite"_.

## How do I pronounce "rew"?

`/ˌriː/` (like _"rewrite"_ but without the _"-write"_)

Hovewer, I personally do not care that much and often just pronounce it simply _"rev"_.

## Why not have a separate binary for each command?

There are several reasons:

1. A single binary is much easier to distribute.
2. Binaries produced by Rust can be quite large due to being statically linked.
   Although there are [ways how to optimize this](https://github.com/johnthagen/min-sized-rust), having just a single binary results in a huge size reduction overall.
3. It is really difficult to come up with reasonable and unique names for new commands.
   A lot of names (I have considered) were already taken by some other commonly used commands.
   And choosing some weird (but unique) names did not seem reasonable.
   Having subcommands means I can give them whatever name I think is appropriate.

If you find this inconvenient, feel free to add aliases for your favorite shell:

```shell
alias trim='rew trim'
alias x='rew x'
...
```
