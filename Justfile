# Search path configuration
tools_dir := join(justfile_directory(), "tools")
path_separator := if os() == "windows" { ";" } else { ":" }
export PATH := tools_dir + path_separator + env("PATH")

# Nextest configuration
export NEXTEST_STATUS_LEVEL := "leak"

# Show available recipes
@default:
    just --list --unsorted

# Development workflow (format, build, lint, test)
dev: format build lint test

# Run with args
run *ARGS:
    cargo run -- {{ARGS}}

# Build
build:
    cargo build

# Install release build to ~/.local/bin/
install:
    cargo build --release
    mkdir -p ~/.local/bin/
    cp target/release/rew ~/.local/bin/

# Format code
format:
    cargo +nightly fmt

# Run linter
lint:
    cargo clippy

# Run tests
test:
    cargo nextest run --no-fail-fast

# Generate code coverage as HTML (and open it)
coverage:
    cargo llvm-cov nextest --json | llvm-cov-pretty --open

# Clean generated files
clean:
    cargo clean

# Set up development environment
setup:
    cargo binstall --install-path=tools --no-confirm \
        cargo-llvm-cov@0.6.16 \
        cargo-nextest@0.9.93 \
        llvm-cov-pretty@0.1.10
