#!/usr/bin/env sh

set -euo pipefail

VERSION=$(grep '^version' Cargo.toml | grep -Po '\d+\.\d+\.\d+')
git tag -am "Version $VERSION" "v$VERSION"
