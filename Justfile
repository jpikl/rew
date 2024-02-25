# Show available recipes
@default:
    just --list --unsorted

# Development workflow (format, clippy, test, docs)
dev: format clippy test docs

# Docs development workflow
dev-docs:
    #!/usr/bin/env -S sh -eux
    mdbook serve --open &
    cargo watch ---ignore docs -- just docs &
    trap 'kill $(jobs -pr)' EXIT
    wait

# Format code
format:
    cargo +nightly fmt --all

# Run clippy
clippy:
    cargo clippy --workspace -- \
        -D clippy::all \
        -D clippy::pedantic \
        -A clippy::module_name_repetitions \
        -A clippy::must_use_candidate

# Run tests
test:
    cargo test --package rew --tests --quiet

# Generate docs
docs:
    cargo run --package xtask -- docs

# Build with release profile
build:
    cargo build --release

# Build and install to ~/.local/bin/
install: build
    mkdir -p ~/.local/bin/
    cp target/release/rew ~/.local/bin/

# Run rew with args
run +ARGS:
    cargo run -- {{ARGS}}

# Run fuzzer
fuzz:
    cargo +nightly fuzz run --jobs {{num_cpus()}} pattern

# Generate code coverage
coverage format:
    cargo tarpaulin \
        --packages rew \
        --engine llvm \
        --force-clean \
        --exclude-files 'tests/*' \
        --exclude-files 'fuzz/*' \
        --exclude-files 'xtask/*' \
        --out {{format}} \
        -- \
        --quiet

# Preview code coverage as HTML
coverage-preview:
    just coverage html
    xdg-open tarpaulin-report.html

# Clean generated files
clean:
    cargo clean
    rm -rf book tarpaulin-report.html

# Set up development environment
[confirm("This might break your environment!\nRun `just --show setup` first to check what it does.\nContinue? [y/n]:")]
setup:
    rustup self update
    rustup install stable
    rustup install nightly
    if [ ! -x "$(command -v cargo-binstall)" ]; then \
        curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash; \
    else \
        cargo binstall cargo-binstall; \
    fi
    cargo binstall mdbook cargo-watch cargo-tarpaulin cargo-fuzz
