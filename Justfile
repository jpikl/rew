# Show available recipes
@default:
    just --list --unsorted

# Development workflow (format, build, clippy, test, docs)
dev: format build clippy test docs

# Docs development workflow
dev-docs:
    #!/usr/bin/env -S sh -eux
    mdbook serve --open &
    cargo watch ---ignore docs -- just docs &
    trap 'kill $(jobs -pr)' EXIT
    wait

# Format code
format *ARGS:
    cargo +nightly fmt --all {{ARGS}}

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

# Build
build:
    cargo build --workspace --exclude fuzz

# Generate docs
docs:
    cargo run --package xtask -- docs

# Install release build to ~/.local/bin/
install:
    cargo build --release
    mkdir -p ~/.local/bin/
    cp target/release/rew ~/.local/bin/

# Run `rew` with args
run *ARGS:
    cargo run -- {{ARGS}}

# Run `rew x` with a pattern
x PATTERN:
    cargo run -- x "{{PATTERN}}"

# Run fuzzer
fuzz:
    cargo +nightly fuzz run --jobs {{num_cpus()}} pattern

# Generate code coverage
coverage format:
    cargo tarpaulin \
        --packages rew \
        --tests \
        --engine llvm \
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
    rm -rf book cobertura.xml tarpaulin-report.html

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
    cargo binstall mdbook coreutils cargo-watch cargo-tarpaulin cargo-fuzz
