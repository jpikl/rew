#!/usr/bin/env bash

set -euo pipefail

VERSION=$(grep '^version' Cargo.toml | grep -Po '\d+\.\d+\.\d+')
git tag -am "Version $VERSION" "v$VERSION"
