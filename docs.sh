#!/usr/bin/env sh

mdbook serve --open & 
cargo watch -x 'xtask docs' -i docs
