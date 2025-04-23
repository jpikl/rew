# Utilities
binary_ext := if os() == "windows" { ".exe" } else { "" }
binary := "rew" + binary_ext

# Default target for release builds
default_target := if os() == "linux" {
    arch() + "-unknown-linux-gnu"
} else if os() == "windows" {
    arch() + "-pc-windows-msvc"
} else if os() == "macos" {
    arch() + "-apple-darwin"
} else {
    arch() + "-unknown-" + os()
}

# Release build configuration
release_rust_flags := "-Zlocation-detail=none -Zfmt-debug=none"
release_build_flags := "-Zbuild-std=std,panic_abort -Zbuild-std-features=panic_immediate_abort"
install_dir := home_dir() / ".local/bin"

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
run *args:
    cargo run -- {{args}}

# Build development version
build:
    cargo build
    @printf "Build output: "
    @du -h target/release/{{binary}} | awk '{print $2 " (" $1 ")"}'

# Build release version
release target=default_target:
    RUSTFLAGS="{{release_rust_flags}}" cargo build {{release_build_flags}} --target {{target}} --release
    @printf "Build output: "
    @du -h target/{{target}}/release/{{binary}} | awk '{print $2 " (" $1 ")"}'

# Show sizes of release build components 
bloat target=default_target:
    RUSTFLAGS="{{release_rust_flags}}" cargo bloat {{release_build_flags}} --target {{target}} --release --crates

# Install release build to ~/.local/bin/
install target=default_target: (release target)
    mkdir -p {{install_dir}}
    cp target/{{target}}/release/{{binary}} {{install_dir}}

# Format code
format *args:
    cargo fmt {{args}}

# Run linter
lint:
    cargo clippy

# Run tests
test:
    cargo nextest run --no-fail-fast

# Run mutants
mutants *args:
    cargo mutants {{args}}

# Generate code coverage as HTML (and open it)
coverage:
    cargo llvm-cov nextest --json | llvm-cov-pretty --open

# Generate code coverage as Codecov JSON
coverage-codecov:
    cargo llvm-cov nextest --codecov --output-path codecov.json

# Clean generated files
clean:
    cargo clean
    rm -rf mutants.out mutants.out.old

# Set up development environment
setup:
    cargo binstall --install-path=tools --no-confirm \
        cargo-bloat@0.12.1 \
        cargo-llvm-cov@0.6.16 \
        cargo-mutants@25.0.0 \
        cargo-nextest@0.9.93 \
        llvm-cov-pretty@0.1.10
