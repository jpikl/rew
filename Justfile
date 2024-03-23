# TODO: Change to 'gh-pages' before the next release.
pages-branch := "gh-pages-dev"
pages-temp-dir := "/tmp/rew/pages"

# Show available recipes
@default:
    just --list --unsorted

# Development workflow (format, build, clippy, test, docs, shellcheck)
dev: format build clippy test docs shellcheck

# Run `rew` with args
run *ARGS:
    cargo run -- {{ARGS}}

# Run `rew x` with a pattern
x PATTERN:
    cargo run -- x '{{PATTERN}}'

# Build
build:
    cargo build --workspace --exclude fuzz

# Install release build to ~/.local/bin/
install:
    cargo build --release
    mkdir -p ~/.local/bin/
    cp target/release/rew ~/.local/bin/

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
    cargo nextest run --status-level leak --no-fail-fast

# Generate code coverage
coverage format:
    cargo tarpaulin \
        --engine llvm \
        --exclude-files 'tests/*' \
        --exclude-files 'fuzz/*' \
        --exclude-files 'xtask/*' \
        --out {{format}}

# Preview code coverage as HTML
coverage-preview:
    just coverage html
    xdg-open tarpaulin-report.html

# Run fuzzer
fuzz:
    cargo +nightly fuzz run --jobs {{num_cpus()}} pattern

# Run benchmarks
bench *ARGS:
    ./benches/run.sh {{ARGS}}

# Generate docs
docs:
    cargo run --package xtask -- docs

# Pages development workflow
pages:
    #!/usr/bin/env -S sh -eux
    mdbook serve --open &
    cargo watch --ignore '*.{md,css,sh,txt}' -- just docs &
    trap 'kill $(jobs -pr)' EXIT
    wait

# Build and deploy pages
pages-deploy:
    mdbook build
    rm -rf '{{pages-temp-dir}}'
    git fetch origin '{{pages-branch}}'
    git worktree prune
    git worktree add '{{pages-temp-dir}}' '{{pages-branch}}'
    cp -rp pages/* '{{pages-temp-dir}}'
    (cd '{{pages-temp-dir}}' && git add --all)
    (cd '{{pages-temp-dir}}' && git commit --amend -m 'Deploy pages')
    git push --force origin '{{pages-branch}}'

# Run shellcheck on scripts
shellcheck:
    shellcheck -xa benches/*.sh benches/commands/*.sh

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
    cargo binstall cargo-fuzz cargo-nextest cargo-tarpaulin cargo-watch mdbook
